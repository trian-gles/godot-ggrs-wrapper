use crate::*;
use gdnative::core_types::ToVariant;
use ggrs::*;
use std::convert::TryInto;
use std::option::*;

/// A Godot implementation of [`P2PSession`]
#[derive(NativeClass)]
#[inherit(Node)]
pub struct GodotGGRSP2PSession {
    sess: Option<P2PSession>,
    callback_node: Option<Ref<Node>>,
    next_handle: usize,
}

impl GodotGGRSP2PSession {
    fn new(_owner: &Node) -> Self {
        GodotGGRSP2PSession {
            sess: None,
            callback_node: None,
            next_handle: 0,
        }
    }
}

#[methods]
impl GodotGGRSP2PSession {
    //EXPORTED FUNCTIONS
    #[export]
    fn _ready(&self, _owner: &Node) {
        godot_print!("GodotGGRSP2PSession _ready() called.");
    }

    /// Creates a GGRSP2PSession
    #[export]
    pub fn create_session(&mut self, _owner: &Node, local_port: u16, num_players: u32) {
        let input_size: usize = std::mem::size_of::<u32>();
        match P2PSession::new(num_players, input_size, local_port) {
            Ok(s) => self.sess = Some(s),
            Err(e) => godot_error!("{}", e),
        }
    }

    #[export]
    fn add_local_player(&mut self, _owner: &Node) -> usize {
        self.add_player(PlayerType::Local)
    }

    #[export]
    fn add_remote_player(&mut self, _owner: &Node, address: String) -> usize {
        let remote_addr: std::net::SocketAddr = address.parse().unwrap();
        self.add_player(PlayerType::Remote(remote_addr))
    }

    #[export]
    fn add_spectator(&mut self, _owner: &Node, address: String) -> usize {
        let remote_addr: std::net::SocketAddr = address.parse().unwrap();
        self.add_player(PlayerType::Spectator(remote_addr))
    }

    #[export]
    fn start_session(&mut self, _owner: &Node) {
        match &mut self.sess {
            Some(s) => match s.start_session() {
                Ok(_) => godot_print!("Started GodotGGRS session"),
                Err(e) => {
                    godot_error!("{}", e);
                }
            },
            None => {
                godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE)
            }
        }
    }

    #[export]
    fn is_running(&mut self, _owner: &Node) -> bool {
        match &mut self.sess {
            Some(s) => s.current_state() == SessionState::Running,
            None => false,
        }
    }

    #[export]
    fn get_current_state(&mut self, _owner: &Node) -> String {
        match &mut self.sess {
            Some(s) => match s.current_state() {
                SessionState::Initializing => "Initializing".to_owned(),
                SessionState::Running => "Running".to_owned(),
                SessionState::Synchronizing => "Synchronizing".to_owned(),
            },
            None => {
                godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE);
                "".to_owned()
            }
        }
    }

    #[export]
    fn advance_frame(&mut self, _owner: &Node, local_player_handle: usize, local_input: u32) {
        //Convert local_input into a byte array
        let local_input_bytes = local_input.to_be_bytes();
        let local_input_array_slice: &[u8] = &local_input_bytes[..];

        match &mut self.sess {
            Some(s) => match s.advance_frame(local_player_handle, local_input_array_slice) {
                Ok(requests) => {
                    self.handle_requests(requests);
                }
                Err(e) => {
                    godot_error!("{}", e);
                }
            },
            None => {
                godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE);
            }
        }
    }

    #[export]
    fn set_fps(&mut self, _owner: &Node, fps: u32) {
        match &mut self.sess {
            Some(s) => match s.set_fps(fps) {
                Ok(_) => return,
                Err(e) => godot_error!("{}", e),
            },
            None => godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE),
        }
    }

    #[export]
    fn receive_callback_node(&mut self, _owner: &Node, callback: Ref<Node>) {
        self.callback_node = Some(callback);
    }

    #[export]
    fn poll_remote_clients(&mut self, _owner: &Node) {
        match &mut self.sess {
            Some(s) => s.poll_remote_clients(),
            None => godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE),
        }
    }

    #[export]
    fn print_network_stats(&mut self, _owner: &Node, handle: PlayerHandle) {
        match &mut self.sess {
            Some(s) => match s.network_stats(handle) {
                Ok(n) => godot_print!("send_queue_len: {0}; ping: {1}; kbps_sent: {2}; local_frames_behind: {3}; remote_frames_behind: {4};", n.send_queue_len, n.ping, n.kbps_sent, n.local_frames_behind, n.remote_frames_behind),
                Err(e) => godot_error!("{}", e),
            },
            None => godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE),
        }
    }

    #[export]
    fn get_network_stats(
        &mut self,
        _owner: &Node,
        handle: PlayerHandle,
    ) -> (usize, u64, usize, i32, i32) {
        const DEFAULT_RESPONSE: (usize, u64, usize, i32, i32) = (0, 0, 0, 0, 0);
        match &mut self.sess {
            Some(s) => match s.network_stats(handle) {
                Ok(n) => (
                    n.send_queue_len,
                    n.ping as u64,
                    n.kbps_sent,
                    n.local_frames_behind,
                    n.remote_frames_behind,
                ),
                Err(e) => {
                    godot_error!("{}", e);
                    DEFAULT_RESPONSE
                }
            },
            None => {
                godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE);
                DEFAULT_RESPONSE
            }
        }
    }

    #[export]
    fn set_frame_delay(&mut self, _owner: &Node, frame_delay: u32, player_handle: PlayerHandle) {
        match &mut self.sess {
            Some(s) => match s.set_frame_delay(frame_delay, player_handle) {
                Ok(_) => return,
                Err(e) => godot_error!("{}", e),
            },
            None => godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE),
        }
    }

    #[export]
    fn set_disconnect_timeout(&mut self, _owner: &Node, secs: u64) {
        match &mut self.sess {
            Some(s) => s.set_disconnect_timeout(std::time::Duration::from_secs(secs)),
            None => godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE),
        }
    }

    #[export]
    fn set_disconnect_notify_delay(&mut self, _owner: &Node, secs: u64) {
        match &mut self.sess {
            Some(s) => s.set_disconnect_notify_delay(std::time::Duration::from_secs(secs)),
            None => godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE),
        }
    }

    #[export]
    fn set_sparse_saving(&mut self, _owner: &Node, sparse_saving: bool) {
        match &mut self.sess {
            Some(s) => match s.set_sparse_saving(sparse_saving) {
                Ok(_) => return,
                Err(e) => godot_error!("{}", e),
            },
            None => godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE),
        }
    }

    #[export]
    fn disconnect_player(&mut self, _owner: &Node, player_handle: PlayerHandle) {
        match &mut self.sess {
            Some(s) => match s.disconnect_player(player_handle) {
                Ok(_) => return,
                Err(e) => godot_error!("{}", e),
            },
            None => godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE),
        }
    }

    #[export]
    fn get_events(&mut self, _owner: &Node) -> Vec<(&str, Variant)> {
        let mut result: Vec<(&str, Variant)> = Vec::new();
        match &mut self.sess {
            Some(s) => {
                for event in s.events() {
                    match event {
                        GGRSEvent::WaitRecommendation { skip_frames } => {
                            result.push(("WaitRecommendation", skip_frames.to_variant()))
                        }
                        GGRSEvent::NetworkInterrupted {
                            player_handle,
                            disconnect_timeout,
                        } => result.push((
                            "NetworkInterrupted",
                            (player_handle, disconnect_timeout as u64).to_variant(),
                        )),
                        GGRSEvent::NetworkResumed { player_handle } => {
                            result.push(("NetworkResumed", player_handle.to_variant()))
                        }
                        GGRSEvent::Disconnected { player_handle } => {
                            result.push(("Disconnected", player_handle.to_variant()))
                        }
                        GGRSEvent::Synchronized { player_handle } => {
                            result.push(("Synchronized", player_handle.to_variant()))
                        }
                        GGRSEvent::Synchronizing {
                            player_handle,
                            total,
                            count,
                        } => result
                            .push(("Synchronizing", (player_handle, total, count).to_variant())),
                    }
                }
            }
            None => godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE),
        };
        return result;
    }

    //NON-EXPORTED FUNCTIONS
    fn handle_requests(&mut self, requests: Vec<GGRSRequest>) {
        for item in requests {
            match item {
                GGRSRequest::AdvanceFrame { inputs } => self.ggrs_request_advance_fame(inputs),
                GGRSRequest::LoadGameState { cell } => self.ggrs_request_load_game_state(cell),
                GGRSRequest::SaveGameState { cell, frame } => {
                    self.ggrs_request_save_game_state(cell, frame);
                }
            }
        }
    }

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

    fn add_player(&mut self, player_type: PlayerType) -> usize {
        match &mut self.sess {
            Some(s) => match s.add_player(player_type, self.next_handle) {
                Ok(o) => {
                    self.next_handle += 1;
                    return o;
                }
                Err(e) => {
                    godot_error!("{}", e);
                    panic!()
                }
            },
            None => {
                godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE);
                panic!()
            }
        };
    }
}
