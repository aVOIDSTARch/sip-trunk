use sip_trunk_core::{CallStore, TelnyxClient, VoiceTextClient};

#[derive(Clone)]
pub struct AppState {
    pub call_store: CallStore,
    pub telnyx: TelnyxClient,
    pub voicetext: VoiceTextClient,
}
