use feu_core::app::App;
use std::net::ToSocketAddrs;

pub async fn serve<E, A>(_app: App<E>, _addr: A) -> feu_core::error::Result<()>
where
    E: Clone + Send + Sync + 'static,
    A: ToSocketAddrs + Send,
{
    // stub
    Ok(())
}
