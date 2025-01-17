use crate::*;
use ggrs::{Frame, GGRSRequest, GameState, GameStateCell, PlayerHandle, SyncTestSession};
use std::convert::TryInto;

/// A Godot implementation of [`SyncTestSession`]
#[derive(NativeClass)]
#[inherit(Node)]
pub struct GodotGGRSSyncTestSession {
    sess: Option<SyncTestSession>,
    callback_node: Option<Ref<Node>>,
}

impl GodotGGRSSyncTestSession {
    fn new(_owner: &Node) -> Self {
        GodotGGRSSyncTestSession {
            sess: None,
            callback_node: None,
        }
    }
}

#[methods]
impl GodotGGRSSyncTestSession {
    //EXPORTED FUNCTIONS
    #[export]
    fn _ready(&self, _owner: &Node) {
        godot_print!("GodotGGRSSyncTest _ready() called.");
    }

    /// Creates a [SyncTestSession],
    /// call this when you want to start setting up a `SyncTestSession` takes the total number of players and the check distance as parameters
    #[export]
    pub fn create_session(&mut self, _owner: &Node, num_players: u32, check_distance: u32) {
        let input_size: usize = std::mem::size_of::<u32>();
        match SyncTestSession::new(num_players, input_size, check_distance) {
            Ok(s) => self.sess = Some(s),
            Err(e) => godot_error!("{}", e),
        }
    }

    /// Sets [SyncTestSession::set_frame_delay()] of specified handle.
    /// # Errors
    /// - Will print an [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[export]
    pub fn set_frame_delay(
        &mut self,
        _owner: &Node,
        frame_delay: u32,
        player_handle: PlayerHandle,
    ) {
        match &mut self.sess {
            Some(s) => match s.set_frame_delay(frame_delay, player_handle) {
                Ok(_) => return,
                Err(e) => godot_error!("{}", e),
            },
            None => godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE),
        }
    }

    /// This function will advance the frame using an array of all the inputs given as a parameter (inputs are currently an int in Godot).
    /// Before using this function you have to set the callback node and make sure it has the following callback functions implemented
    /// - [CALLBACK_FUNC_SAVE_GAME_STATE]
    /// - [CALLBACK_FUNC_LOAD_GAME_STATE]
    /// - [CALLBACK_FUNC_SAVE_GAME_STATE]
    /// # Errors
    /// - Will print an [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    /// - Will print an [ERR_MESSAGE_NO_CALLBACK_NODE] error if a callback node has not been set
    #[export]
    pub fn advance_frame(&mut self, _owner: &Node, all_inputs: Vec<u32>) {
        let mut all_inputs_bytes = Vec::new();
        for i in all_inputs {
            all_inputs_bytes.push(Vec::from(i.to_be_bytes()));
        }

        match &mut self.sess {
            Some(s) => match s.advance_frame(&all_inputs_bytes) {
                Ok(requests) => self.handle_requests(requests),
                Err(e) => {
                    godot_error!("{}", e)
                }
            },
            None => {
                godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE)
            }
        }
    }

    /// Sets the callback node that will be called when using [Self::advance_frame()]
    #[export]
    pub fn set_callback_node(&mut self, _owner: &Node, callback: Ref<Node>) {
        self.callback_node = Some(callback);
    }

    //NON-EXPORTED FUNCTIONS
    fn handle_requests(&mut self, requests: Vec<GGRSRequest>) {
        for item in requests {
            match item {
                GGRSRequest::AdvanceFrame { inputs } => self.ggrs_request_advance_fame(inputs),
                GGRSRequest::LoadGameState { cell } => self.ggrs_request_load_game_state(cell),
                GGRSRequest::SaveGameState { cell, frame } => {
                    self.ggrs_request_save_game_state(cell, frame)
                }
            }
        }
    }
    ////GGRSRequest handlers
    fn ggrs_request_advance_fame(&self, inputs: Vec<ggrs::GameInput>) {
        //Parse parameter inputs in a way that godot can handle then call the callback method
        match self.callback_node {
            Some(s) => {
                let node = unsafe { s.assume_safe() };
                let mut godot_array: Vec<Variant> = Vec::new();
                for i in inputs {
                    let result = (
                        i.frame,
                        i.size,
                        u32::from_be_bytes(
                            i.buffer[..i.size]
                                .try_into()
                                .expect("Slice size is too big or too small to convert into u32"),
                        ),
                    )
                        .to_variant();
                    godot_array.push(result);
                }
                unsafe { node.call(CALLBACK_FUNC_ADVANCE_FRAME, &[godot_array.to_variant()]) };
            }
            None => {
                godot_error!("{}", ERR_MESSAGE_NO_CALLBACK_NODE);
            }
        }
    }

    fn ggrs_request_load_game_state(&self, cell: GameStateCell) {
        //Unpack the cell and have over it's values to godot so it can handle it.
        match self.callback_node {
            Some(s) => {
                let node = unsafe { s.assume_safe() };
                let game_state = cell.load();
                let frame = game_state.frame.to_variant();
                let buffer =
                    ByteArray::from_vec(game_state.buffer.unwrap_or_default()).to_variant();
                let checksum = game_state.checksum.to_variant();
                unsafe { node.call(CALLBACK_FUNC_LOAD_GAME_STATE, &[frame, buffer, checksum]) };
            }
            None => {
                godot_error!("{}", ERR_MESSAGE_NO_CALLBACK_NODE);
            }
        }
    }

    fn ggrs_request_save_game_state(&mut self, cell: GameStateCell, frame: Frame) {
        //Store current cell for later use
        match self.callback_node {
            Some(s) => {
                let node = unsafe { s.assume_safe() };
                let state: Variant =
                    unsafe { node.call(CALLBACK_FUNC_SAVE_GAME_STATE, &[frame.to_variant()]) };
                let state_bytes = ByteArray::from_variant(&state).unwrap_or_default();
                let mut state_bytes_vec = Vec::new();
                for i in 0..state_bytes.len() {
                    state_bytes_vec.push(state_bytes.get(i));
                }
                let result = GameState::new(frame, Some(state_bytes_vec), None);
                cell.save(result);
            }
            None => {
                godot_error!("{}", ERR_MESSAGE_NO_CALLBACK_NODE);
            }
        }
    }
}
