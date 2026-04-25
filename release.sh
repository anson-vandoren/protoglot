#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PACKAGE="protoglot"
BINARY_NAME="protoglot"
CARGO_TOML="Cargo.toml"
CHANGELOG_MD="CHANGELOG.md"
MAKE_RELEASE=false

usage() {
    cat << EOF
Usage: $0 [OPTION]

Release protoglot based on CHANGELOG.md and Cargo.toml.

OPTIONS:
    --make-release      create a new release version interactively
    -h, --help          display this help and exit

By default, releases the current version from Cargo.toml after validating it
matches CHANGELOG.md. With --make-release, prompts for a new version, updates
both files, commits changes, pushes the branch, and then proceeds with release.

The release asset is target/release/protoglot.gz plus a SHA256 checksum.

EOF
    exit 1
}

for arg in "$@"; do
    case "$arg" in
        --make-release)
            MAKE_RELEASE=true
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo -e "${RED}❌ Error: unknown option '$arg'${NC}"
            usage
            ;;
    esac
done

get_cargo_version() {
    grep '^version = ' "$CARGO_TOML" | head -1 | sed 's/version = "\(.*\)"/\1/'
}

get_changelog_version() {
    grep -E '^## \[[0-9]+\.[0-9]+\.[0-9]+\]' "$CHANGELOG_MD" | head -1 | sed 's/^## \[\([0-9]*\.[0-9]*\.[0-9]*\)\].*/\1/'
}

get_release_notes() {
    local version="$1"
    awk "/^## \[$version\]/ {found=1; next} found && /^## / {exit} found {print}" "$CHANGELOG_MD" | sed '/^$/d'
}

version_greater() {
    local v1="$1"
    local v2="$2"
    local i n1 n2

    IFS='.' read -ra V1 <<< "$v1"
    IFS='.' read -ra V2 <<< "$v2"

    for i in 0 1 2; do
        n1=${V1[i]:-0}
        n2=${V2[i]:-0}
        if (( n1 > n2 )); then
            return 0
        elif (( n1 < n2 )); then
            return 1
        fi
    done
    return 1
}

increment_patch() {
    local version="$1"
    IFS='.' read -ra V <<< "$version"
    echo "${V[0]}.${V[1]}.$((V[2] + 1))"
}

check_unreleased_changes() {
    local unreleased_content
    unreleased_content=$(awk '/^## \[Unreleased\]/ {found=1; next} found && /^## / {exit} found {print}' "$CHANGELOG_MD")

    if ! echo "$unreleased_content" | grep -q '^### '; then
        echo -e "${RED}❌ CHANGELOG.md has no changes under ## [Unreleased]${NC}"
        echo "Add at least one ### heading with changes before creating a release."
        return 1
    fi
    return 0
}

update_cargo_version() {
    local new_version="$1"
    sed -i "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
}

update_changelog() {
    local new_version="$1"
    local current_date prev_version comparison_link temp_file

    current_date=$(date '+%Y-%m-%d')
    prev_version=$(get_changelog_version)

    if [[ -n "$prev_version" ]]; then
        comparison_link="https://github.com/anson-vandoren/protoglot/compare/v${prev_version}..v${new_version}"
    else
        comparison_link="https://github.com/anson-vandoren/protoglot"
    fi

    temp_file=$(mktemp)
    awk '/^## \[Unreleased\]/ {print; print ""; exit} {print}' "$CHANGELOG_MD" > "$temp_file"

    echo "## [${new_version}](${comparison_link}) - ${current_date}" >> "$temp_file"
    echo "" >> "$temp_file"

    awk '
    /^## \[Unreleased\]/ { found=1; next }
    found && /^## / { exit }
    found && NF > 0 {
        if (/^### /) {
            if (in_section) print ""
            print $0
            print ""
            in_section = 1
        } else if (/^- /) {
            print $0
        } else if (NF > 0) {
            print $0
        }
    }' "$CHANGELOG_MD" >> "$temp_file"

    echo "" >> "$temp_file"
    awk '/^## \[Unreleased\]/ {found=1; next} found && /^## / {print_rest=1} print_rest {print}' "$CHANGELOG_MD" >> "$temp_file"
    mv "$temp_file" "$CHANGELOG_MD"
}

validate_changelog_content() {
    local version="$1"
    local notes
    notes=$(get_release_notes "$version")

    if ! echo "$notes" | grep -q '^### '; then
        echo -e "${RED}CHANGELOG.md for version $version has no subsections (### headings)${NC}"
        return 1
    fi

    if ! echo "$notes" | grep -q '^- '; then
        echo -e "${RED}CHANGELOG.md for version $version has no bullet points (- items)${NC}"
        return 1
    fi

    return 0
}

sha256_file() {
    local file="$1"
    local checksum_file="$2"

    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$file" | awk '{print $1}' > "$checksum_file"
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$file" | awk '{print $1}' > "$checksum_file"
    else
        echo -e "${RED}❌ No SHA256 utility found${NC}"
        exit 1
    fi
}

if [[ "$MAKE_RELEASE" == "true" ]]; then
    echo -e "${BLUE}🚀 Interactive release creation${NC}"

    if [[ -n $(git status --porcelain) ]]; then
        echo -e "${RED}❌ Working directory is not clean. Commit your changes first.${NC}"
        exit 1
    fi

    check_unreleased_changes

    current_cargo_version=$(get_cargo_version)
    current_changelog_version=$(get_changelog_version)
    default_version=$(increment_patch "$current_cargo_version")

    echo -e "${YELLOW}📋 Current version information:${NC}"
    echo "  Cargo.toml:   $current_cargo_version"
    echo "  CHANGELOG.md: $current_changelog_version"
    echo ""
    echo -e "${YELLOW}🔢 Enter new version number (default: $default_version):${NC}"
    read -r new_version

    if [[ -z "$new_version" ]]; then
        new_version="$default_version"
    fi

    if [[ ! "$new_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo -e "${RED}❌ Version must be in format X.Y.Z${NC}"
        exit 1
    fi

    if ! version_greater "$new_version" "$current_cargo_version"; then
        echo -e "${RED}❌ New version $new_version must be greater than Cargo.toml version $current_cargo_version${NC}"
        exit 1
    fi

    if [[ -n "$current_changelog_version" ]] && ! version_greater "$new_version" "$current_changelog_version"; then
        echo -e "${RED}❌ New version $new_version must be greater than CHANGELOG.md version $current_changelog_version${NC}"
        exit 1
    fi

    echo -e "${YELLOW}📝 Updating Cargo.toml and CHANGELOG.md${NC}"
    update_cargo_version "$new_version"
    update_changelog "$new_version"

    echo ""
    echo -e "${BLUE}📋 Changes for version $new_version:${NC}"
    echo "======================================================================================================"
    get_release_notes "$new_version"
    echo "======================================================================================================"
    echo ""
    echo -e "${YELLOW}❓ Proceed with these changes? (y/N)${NC}"
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        echo -e "${RED}❌ Aborted${NC}"
        exit 1
    fi

    git add "$CHANGELOG_MD" "$CARGO_TOML" Cargo.lock
    commit_message="release $PACKAGE $new_version"
    echo -e "${YELLOW}❓ Create commit with message '$commit_message'? (y/N)${NC}"
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        echo -e "${RED}❌ Aborted${NC}"
        exit 1
    fi

    git commit -m "$commit_message"

    current_branch=$(git branch --show-current)
    echo -e "${YELLOW}❓ Push changes to remote '$current_branch'? (y/N)${NC}"
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        echo -e "${RED}❌ Changes committed locally but not pushed. Run 'git push origin $current_branch' manually.${NC}"
        exit 1
    fi

    git push origin "$current_branch"
fi

CARGO_VERSION=$(get_cargo_version)
CHANGELOG_VERSION=$(get_changelog_version)
VERSION="v$CARGO_VERSION"

echo -e "${BLUE}📋 Validating release configuration${NC}"

if [[ ! "$CARGO_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}❌ Cargo.toml version '$CARGO_VERSION' must be in format X.Y.Z${NC}"
    exit 1
fi

if [[ ! -r "$CHANGELOG_MD" ]]; then
    echo -e "${RED}❌ CHANGELOG.md not found or not readable${NC}"
    exit 1
fi

if ! grep -q '^## \[Unreleased\]' "$CHANGELOG_MD"; then
    echo -e "${RED}❌ CHANGELOG.md missing '## [Unreleased]' section${NC}"
    exit 1
fi

if [[ "$CARGO_VERSION" != "$CHANGELOG_VERSION" ]]; then
    echo -e "${RED}❌ Version mismatch:${NC}"
    echo "  Cargo.toml:   $CARGO_VERSION"
    echo "  CHANGELOG.md: $CHANGELOG_VERSION"
    exit 1
fi

validate_changelog_content "$CARGO_VERSION"

echo -e "${GREEN}✅ Version validation passed: $VERSION${NC}"

if [[ -n $(git status --porcelain) ]]; then
    echo -e "${RED}❌ Working directory is not clean. Commit your changes first.${NC}"
    exit 1
fi

CURRENT_BRANCH=$(git branch --show-current)
if [[ "$CURRENT_BRANCH" != "main" ]]; then
    echo -e "${YELLOW}⚠️  Not on main branch (currently on $CURRENT_BRANCH). Continue? (y/N)${NC}"
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo -e "${YELLOW}📥 Pulling latest changes and tags${NC}"
git pull origin "$CURRENT_BRANCH"
git fetch --tags

if git tag -l | grep -q "^${VERSION}$"; then
    echo -e "${RED}❌ Tag ${VERSION} already exists locally${NC}"
    exit 1
fi

if git ls-remote --tags origin | grep -q "refs/tags/${VERSION}$"; then
    echo -e "${RED}❌ Tag ${VERSION} already exists on remote${NC}"
    exit 1
fi

echo -e "${YELLOW}🧪 Running tests${NC}"
if command -v mise >/dev/null 2>&1; then
    mise run test
else
    cargo test --verbose
fi

echo -e "${YELLOW}🔍 Running clippy${NC}"
if command -v mise >/dev/null 2>&1; then
    mise run clippy
else
    cargo clippy --all-targets --all-features
fi

echo -e "${YELLOW}🔨 Building release binary${NC}"
cargo build --release --bin "$BINARY_NAME"

BINARY_PATH="target/release/${BINARY_NAME}"
if [[ ! -f "$BINARY_PATH" ]]; then
    echo -e "${RED}❌ Binary not found at ${BINARY_PATH}${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Binary built successfully: $(ls -lh "$BINARY_PATH" | awk '{print $5}')${NC}"

GZIP_NAME="${PACKAGE}.gz"
FINAL_ASSET_PATH="target/release/${GZIP_NAME}"
CHECKSUM_FILE="${FINAL_ASSET_PATH}.sha256"

echo -e "${YELLOW}🗜️  Compressing binary${NC}"
gzip -c "$BINARY_PATH" > "$FINAL_ASSET_PATH"

echo -e "${YELLOW}🔏 Generating SHA256 checksum${NC}"
sha256_file "$FINAL_ASSET_PATH" "$CHECKSUM_FILE"
echo -e "${GREEN}✅ Checksum generated: $(cat "$CHECKSUM_FILE")${NC}"

RELEASE_NOTES=$(get_release_notes "$CARGO_VERSION")

echo -e "${YELLOW}🏷️  Creating signed git tag ${VERSION}${NC}"
git tag -s "$VERSION" -m "Release $VERSION"
git push origin "$VERSION"

if ! command -v gh >/dev/null 2>&1; then
    echo -e "${RED}❌ GitHub CLI (gh) is required but not installed${NC}"
    echo "Install it with https://cli.github.com/manual/installation"
    exit 1
fi

if ! gh auth status >/dev/null 2>&1; then
    echo -e "${YELLOW}🔐 Please log in to GitHub CLI${NC}"
    gh auth login
fi

echo -e "${YELLOW}📦 Creating GitHub release${NC}"
gh release create "$VERSION" \
    --title "$VERSION" \
    --notes "$RELEASE_NOTES" \
    "$FINAL_ASSET_PATH#${GZIP_NAME}" \
    "$CHECKSUM_FILE#${GZIP_NAME}.sha256"

echo -e "${GREEN}✅ Release $VERSION created successfully${NC}"
