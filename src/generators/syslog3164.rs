use super::EventGenerator;

const MESSAGES: &[&[u8]] = &[
b"<67>Feb 16 02:00:41 crona7704 eos[8025]: We need to reboot the haptic SDD pixel!\n",
b"<15>Feb 16 02:00:41 heaney5416 consequatur[8154]: Try to back up the CSS array, maybe it will quantify the bluetooth alarm!\n",
b"<102>Feb 16 02:00:41 monahan3471 et[9774]: The JBOD system is down, override the auxiliary program so we can input the THX bandwidth!\n",
b"<26>Feb 16 02:00:41 harvey3385 accusantium[6757]: Try to program the HTTP firewall, maybe it will reboot the cross-platform array!\n",
b"<63>Feb 16 02:00:41 kunze2282 voluptatem[6210]: We need to compress the virtual SSL circuit!\n",
b"<118>Feb 16 02:00:41 upton3286 eum[2753]: Try to transmit the SAS feed, maybe it will compress the open-source feed!\n",
b"<118>Feb 16 02:00:41 morar6348 unde[3125]: If we input the sensor, we can get to the AGP sensor through the auxiliary SDD array!\n",
b"<166>Feb 16 02:00:41 willms2506 omnis[2099]: Try to compress the CSS bus, maybe it will navigate the redundant monitor!\n",
b"<166>Feb 16 02:00:41 harvey8852 qui[1420]: You can't generate the array without synthesizing the auxiliary IB sensor!\n",
b"<155>Feb 16 02:00:41 lehner6525 delectus[7212]: Try to parse the PCI array, maybe it will synthesize the 1080p feed!\n",
b"<190>Feb 16 02:00:41 gutkowski4676 rerum[4214]: If we index the protocol, we can get to the SSL system through the haptic CSS bandwidth!\n",
];
pub struct Syslog3164EventGenerator {
    message_index: u64,
}

impl Syslog3164EventGenerator {
    pub fn new() -> Self {
        Self { message_index: 0 }
    }
}

impl EventGenerator for Syslog3164EventGenerator {
    fn generate_into(&mut self, buf: &mut Vec<u8>) {
        self.message_index += 1;
        buf.extend_from_slice(MESSAGES[self.message_index as usize % MESSAGES.len()]);
    }
}
