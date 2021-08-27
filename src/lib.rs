use gdnative::prelude::*;
use std::option::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct GodotGGRS {
    sess: Option<ggrs::P2PSession>,
    callback_nodepath: String,
    next_handle: usize,
}

impl GodotGGRS {
    fn new(_owner: &Node) -> Self {
        GodotGGRS {
            sess: None,
            callback_nodepath: "../.".to_string(),
            next_handle: 0,
        }
    }
}

#[methods]
impl GodotGGRS {
    #[export]
    fn _ready(&self, _owner: &Node) {
        godot_print!("GodotGGRS _ready() called.");
    }

    #[export]
    fn create_session(&mut self, _owner: &Node, local_port: u16, num_players: u32) {
        let input_size: usize = std::mem::size_of::<u32>();
        match ggrs::start_p2p_session(num_players, input_size, local_port) {
            Ok(s) => self.sess = Some(s),
            Err(e) => godot_print!("{:?}", e),
        }
    }

    #[export]
    fn add_local_player(&mut self, _owner: &Node) -> usize {
        self.add_player(ggrs::PlayerType::Local)
    }

    #[export]
    fn add_remote_player(&mut self, _owner: &Node, address: String) -> usize {
        let remote_addr: std::net::SocketAddr = address.parse().unwrap();
        self.add_player(ggrs::PlayerType::Remote(remote_addr))
    }

    #[export]
    fn add_spectator(&mut self, _owner: &Node, address: String) -> usize {
        let remote_addr: std::net::SocketAddr = address.parse().unwrap();
        self.add_player(ggrs::PlayerType::Spectator(remote_addr))
    }

    fn add_player(&mut self, player_type: ggrs::PlayerType) -> usize {
        match &mut self.sess {
            Some(s) => match s.add_player(player_type, self.next_handle) {
                Ok(o) => {
                    self.next_handle += 1;
                    return o;
                }
                Err(e) => {
                    godot_print!("{}", e);
                    panic!()
                }
            },
            None => {
                godot_print!("No session was made.");
                panic!()
            }
        };
    }

    #[export]
    fn receive_callback_node(&mut self, _owner: &Node, nodepath: String) {
        self.callback_nodepath = nodepath;
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<GodotGGRS>();
}

godot_init!(init);
