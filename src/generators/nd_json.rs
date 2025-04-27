use super::EventGenerator;

pub struct NdJsonEventGenerator {
    message_index: u64,
}

impl NdJsonEventGenerator {
    pub fn new() -> Self {
        Self { message_index: 0 }
    }
}

const MESSAGES: &[&[u8]] = &[
b"{\"timestamp\":\"2025-04-27T00:12:49.030Z\",\"hostname\":\"example.com\",\"app_name\":\"example\",\"pid\":2623,\"message\":\"idx=0, uuid=b06983ae-6048-45ff-bc62-51959cb13e36, msg=Dicing Models\"}\n",
b"{\"timestamp\":\"2025-04-27T00:12:49.031Z\",\"hostname\":\"example.com\",\"app_name\":\"example\",\"pid\":4141,\"message\":\"idx=1, uuid=a4e5dabf-c188-4a92-95e5-cc12cf8329bd, msg=Binding Sapling Root System\"}\n",
b"{\"timestamp\":\"2025-04-27T00:12:49.031Z\",\"hostname\":\"example.com\",\"app_name\":\"example\",\"pid\":8791,\"message\":\"idx=2, uuid=23050320-8a12-495b-a125-fd3b271459ad, msg=Initializing My Sim Tracking Mechanism\"}\n",
b"{\"timestamp\":\"2025-04-27T00:12:49.031Z\",\"hostname\":\"example.com\",\"app_name\":\"example\",\"pid\":8393,\"message\":\"idx=3, uuid=1c71c06e-7ebf-4e72-9624-9309e4e53aaa, msg=Aesthesizing Industrial Areas\"}\n",
b"{\"timestamp\":\"2025-04-27T00:12:49.031Z\",\"hostname\":\"example.com\",\"app_name\":\"example\",\"pid\":1383,\"message\":\"idx=4, uuid=6c441f10-5cc8-409f-a2fd-eed77b763e8b, msg=Graphing Whale Migration\"}\n",
b"{\"timestamp\":\"2025-04-27T00:12:49.031Z\",\"hostname\":\"example.com\",\"app_name\":\"example\",\"pid\":8961,\"message\":\"idx=5, uuid=e5fdd0a7-719d-41c2-b547-5627f8bf6f2a, msg=Reconfiguring User Mental Processes\"}\n",
b"{\"timestamp\":\"2025-04-27T00:12:49.031Z\",\"hostname\":\"example.com\",\"app_name\":\"example\",\"pid\":9690,\"message\":\"idx=6, uuid=8b46bea4-f7e8-4ac8-9152-a09651d8b7d0, msg=Projecting Law Enforcement Pastry Intake\"}\n",
b"{\"timestamp\":\"2025-04-27T00:12:49.031Z\",\"hostname\":\"example.com\",\"app_name\":\"example\",\"pid\":8242,\"message\":\"idx=7, uuid=212018b1-2bd8-4e58-90d9-032e96175ffa, msg=Resolving GUID Conflict\"}\n",
b"{\"timestamp\":\"2025-04-27T00:12:49.031Z\",\"hostname\":\"example.com\",\"app_name\":\"example\",\"pid\":2227,\"message\":\"idx=8, uuid=9c5693c9-9bc3-4eac-a2af-2e5d2feac168, msg=Graphing Whale Migration\"}\n",
b"{\"timestamp\":\"2025-04-27T00:12:49.031Z\",\"hostname\":\"example.com\",\"app_name\":\"example\",\"pid\":8479,\"message\":\"idx=9, uuid=3da2cf3e-59eb-4593-8cbe-039fa838a8d7, msg=Preparing Sprites for Random Walks\"}\n"
];

impl EventGenerator for NdJsonEventGenerator {
    fn generate_bytes(&mut self) -> Vec<u8> {
        self.message_index += 1;
        MESSAGES[self.message_index as usize % MESSAGES.len()].to_vec()
    }
}
