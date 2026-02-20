
// fetch server linked to user
#[get("/servers")]
pub async fn get_servers(){}

//fetch server details
#[get("/server/{server}")]
pub async fn get_server(){}

// list all channels in server
#[get("/server/channels")]
pub async fn get_channels(){}

// get details about channel
#[get("/server/{server}/channel/{channel}")]
pub async fn get_channel(){}

// cursor paginadted endpoint via timestamp
#[get("/server/{server}/channel/{channel}/messages")]
pub async fn get_channel_messages(){}

