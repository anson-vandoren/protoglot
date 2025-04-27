use super::EventGenerator;

pub struct Syslog5424EventGenerator {
    message_index: u64,
}

impl Syslog5424EventGenerator {
    pub fn new() -> Self {
        Self { message_index: 0 }
    }
}

const MESSAGES: &[&[u8]] = &[
b"<139>1 2022-10-17T23:47:54.806823Z costume7526 silverCamera.ps1 30521 NiceMamaPhrase [Enjoy Wolf=\"lazarus\" Adrian=\"magnet\"] Initializing my sim tracking mechanism and reverse engineering image consultant\n",
b"<144>1 2022-10-17T23:47:54.806835Z amber1740 neuronLazarus.sh 66217 DecideBalletCity [Drama Benefit=\"38213\"] Compressing fish files so increasing magmafacation\n",
b"<160>1 2022-10-17T23:47:54.806847Z plume6861 legalTextile.sh 45877 RicardoTictacRavioli [Inside Yogurt=\"30642\" Sabrina=\"acrobat\" Symbol=\"heart\" Chant=\"97386\"] Dicing models so integrating curves\n",
b"<152>1 2022-10-17T23:47:54.806859Z almond3615 swingSilence.ps1 87830 LithiumTelexNylon [North Tina=\"65740\"] Debunching unionized commercial services while searching for llamas\n",
b"<66>1 2022-10-17T23:47:54.806873Z epoxy7153 italianAndroid.jar 31870 ThomasAbsurdJordan [Turbo Laser=\"83020\" Metal=\"57010\" Raymond=\"49962\" Mister=\"69501\" Labor=\"ruby\"] Setting inner deity indicators or calculating llama expectoration trajectory\n",
b"<123>1 2022-10-17T23:47:54.806882Z ironic2625 marcoCasino.jar 69539 MemphisVampireFrame [Fame Easy=\"86271\" Gorilla=\"plume\" Value=\"hello\"] Deleting ferry routes and compounding inert tessellations\n",
b"<28>1 2022-10-17T23:47:54.806896Z gemini6109 agendaExact.ps1 97191 FactorPeaceChamber [Kansas Fossil=\"71652\"] Calibrating blue skies while deciding what message to display next\n",
b"<105>1 2022-10-17T23:47:54.806907Z trilogy5929 junePyramid.bat 27408 PreferAnalogHarris [Citizen Amigo=\"green\" Rent=\"paul\" Strange=\"sierra\" Diego=\"88578\" Natasha=\"exodus\"] Adding hidden agendas while partitioning city grid singularities\n",
b"<13>1 2022-10-17T23:47:54.806916Z soprano5162 greenSharon.exe 9889 PercentDecideAbsent [Recycle Audio=\"97486\" Gemini=\"87909\" Spark=\"nobody\"] Aligning covariance matrices yet implementing impeachment routine\n",
b"<143>1 2022-10-17T23:47:54.806929Z riviera2444 filmSnow.sh 98408 NorwayOthelloData [Balance Morph=\"30355\" Lady=\"first\"] Collecting meteor particles so prioritizing landmarks\n",
b"<103>1 2022-10-17T23:47:54.806940Z ecology4557 spiralRapid.sh 13160 MagnetMorningScarlet [Include Benefit=\"18501\" List=\"3574\" Radar=\"switch\" Uncle=\"35252\" Honey=\"mozart\"] Sequencing particles so stratifying ground layers\n",
b"<127>1 2022-10-17T23:47:54.806950Z rebel1201 tavernPermit.bat 37738 QuarterLectureSalmon [Organic Traffic=\"edition\" Pacific=\"ginger\" Century=\"73901\"] Integrating curves while preparing sprites for random walks\n",
b"<140>1 2022-10-17T23:47:54.806964Z manual7579 genuinePolka.py 10924 NatashaTavernJackson [Solid Uncle=\"40873\"] Depositing slush funds or removing vehicle avoidance behavior\n",
b"<141>1 2022-10-17T23:47:54.806975Z special8484 weatherMorning.cmd 81067 SalamiMarbleCartel [Chaos Flood=\"shake\"] Determining width of blast fronts so searching for llamas\n",
b"<142>1 2022-10-17T23:47:54.806983Z trust7610 duetSlogan.bat 11222 FirstDesignSweet [Answer Ozone=\"viva\" October=\"7412\"] Normalizing power and time-compressing simulator clock\n",
b"<27>1 2022-10-17T23:47:54.806992Z visible5582 borisPassive.jar 5316 DiamondTarzanGold [Voyage Square=\"61054\"] Stratifying ground layers and extracting resources\n",
b"<74>1 2022-10-17T23:47:54.807001Z joker4518 focusEthnic.jar 66917 SodaAmigoSummer [Silk Danube=\"74487\"] Bureacritizing bureaucracies so downloading satellite terrain data\n",
b"<113>1 2022-10-17T23:47:54.807010Z trumpet9874 evitaShine.bat 97095 ClaudiaMonitorRobin [Ricardo Bicycle=\"nobel\"] Resolving guid conflict so gathering particle sources\n",
b"<30>1 2022-10-17T23:47:54.807019Z jessica5241 dependAtomic.py 12978 EdwardCompanyGrille [Honey Danube=\"morph\" Gizmo=\"96303\" Charlie=\"14888\"] Prioritizing landmarks so flood-filling ground water\n",
b"<21>1 2022-10-17T23:47:54.807027Z sting1909 passagePassive.ps1 67097 SolarPlasmaPasta [Jeep Visa=\"29267\" Europe=\"juliet\" Planet=\"letter\" Glass=\"61989\"] Deunionizing bulldozers but determining width of blast fronts\n",
b"<119>1 2022-10-17T23:47:54.807038Z nadia9651 episodePromise.jar 17573 RemarkHandSafari [Gregory Anvil=\"metal\" Import=\"percent\" Goblin=\"double\" Pedro=\"video\"] Zeroing crime network so concatenating sub-contractors\n",
b"<106>1 2022-10-17T23:47:54.807052Z combat1560 garageWheel.cmd 26115 MixerAvenuePolka [Voltage Random=\"prime\" Sahara=\"poker\" Maestro=\"invest\"] Applying feng shui shaders after prioritizing landmarks\n",
b"<39>1 2022-10-17T23:47:54.807064Z aloha5182 opticTemple.jar 3682 NirvanaShelfNatasha [Frozen Nina=\"72611\" Before=\"79532\"] Iterating cellular automata and time-compressing simulator clock\n",
b"<174>1 2022-10-17T23:47:54.807075Z flipper3754 obscureActive.cmd 54854 PilotSilenceJumbo [Ritual Fast=\"spray\" Star=\"53893\" Brown=\"jacob\" Partner=\"salami\" Quarter=\"73621\"] Determining width of blast fronts or applying feng shui shaders\n",
b"<135>1 2022-10-17T23:47:54.807085Z duet9779 vincentResume.py 93726 NinjaFolioBicycle [Gold Diego=\"mega\" Classic=\"manager\"] Initializing rhinoceros breeding timetable but initializing my sim tracking mechanism\n",
b"<147>1 2022-10-17T23:47:54.807099Z catalog6483 teacherPolice.exe 27829 BlitzMobileDecade [Zoom Farmer=\"52258\" Trick=\"poker\" Parole=\"12428\"] Zeroing crime network while building data trees\n",
b"<176>1 2022-10-17T23:47:54.807108Z ego7380 telecomTeacher.py 36939 BuzzerGeminiVoltage [Split Mental=\"quiz\" Film=\"mama\" Pigment=\"1050\" Juice=\"37334\"] Deciding what message to display next and removing vehicle avoidance behavior\n",
b"<144>1 2022-10-17T23:47:54.807119Z beach5027 percentGabriel.exe 59488 FantasyIncaSolo [Uranium Campus=\"passage\"] Polishing water highlights so projecting law enforcement pastry intake\n",
b"<156>1 2022-10-17T23:47:54.807131Z system6752 emeraldPlaza.jar 8636 SalmonAthleteIndex [Tiger Parent=\"45516\" Mirror=\"music\"] Calibrating blue skies but cohorting exemplars\n",
b"<105>1 2022-10-17T23:47:54.807139Z sport8024 judgeRover.jar 78361 ZebraValeryStand [Tunnel Input=\"chief\" Detect=\"greek\" John=\"16280\" Axiom=\"table\"] Setting advisor moods but preparing sprites for random walks\n",
];

impl EventGenerator for Syslog5424EventGenerator {
    fn generate_bytes(&mut self) -> Vec<u8> {
        // add a \n to the end of the message
        self.message_index += 1;
        MESSAGES[self.message_index as usize % MESSAGES.len()].to_vec()
    }
}
