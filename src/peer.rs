//ces quoi un peer ces du network ?


pub struct Peer{
    addr : Vec<SocketAddr>,
    last_view : Vec<u64>,
}


impl Peer{
    pub fn get_addr(&self) -> Vec<SocketAddr>{
        self.addr;
    }

    pub fn push(&self, addr : SocketAddr, time :u64   ) {


    }

}