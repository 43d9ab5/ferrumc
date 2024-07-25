use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::OnceCell;
use tracing::info;

use base64::Engine;
use ferrumc_macros::{Decode, packet};

use crate::Connection;
use crate::net::packets::IncomingPacket;
use crate::net::packets::outgoing::status::OutgoingStatusResponse;
use crate::utils::config;
use crate::utils::encoding::varint::VarInt;
use crate::utils::prelude::*;
use crate::utils::type_impls::Encode;

/// The status packet is sent by the client to the server to request the server's status.
///
/// Usually sent after handshaking is completed.
#[derive(Decode)]
#[packet(packet_id = 0x00, state = "status")]
pub struct Status;

/// The response to the status packet.
/// Sent as json.
#[derive(Serialize)]
struct JsonResponse {
    version: Version,
    players: Players,
    description: Description,
    favicon: &'static String,
}

#[derive(Serialize)]
struct Version {
    name: String,
    protocol: u32,
}

#[derive(Serialize)]
struct Players {
    max: u32,
    online: u32,
    sample: Vec<Sample>,
}

#[derive(Serialize)]
struct Sample {
    name: String,
    id: String,
}

#[derive(Serialize)]
struct Description {
    text: String,
}

impl IncomingPacket for Status {
    async fn handle(&self, conn: &mut Connection) -> Result<()> {
        info!("Handling status request packet");
        let config = config::get_global_config();

        let response = OutgoingStatusResponse {
            packet_id: VarInt::new(0x00),
            json_response: serde_json::ser::to_string(&JsonResponse {
                version: Version {
                    name: "1.20.6".to_string(),
                    // Allow any protocol version for now. To check the ping and stuff
                    protocol: conn.metadata.protocol_version.clone() as u32,
                },
                players: Players {
                    max: config.max_players,
                    online: 2,
                    sample: vec![
                        Sample {
                            name: "Recore_".to_string(),
                            id: "2b3414ed-468a-45c2-b113-6c5f47430edc".to_string(),
                        },
                        Sample {
                            name: "sweattypalms".to_string(),
                            id: "26d88d10-f052-430f-9406-e6c3089792c4".to_string(),
                        },
                    ],
                },
                description: Description {
                    text: config.motd.clone(),
                },
                favicon: get_encoded_favicon().await,
            })
                .unwrap(),
        };

        let mut cursor = std::io::Cursor::new(Vec::new());
        response.encode(&mut cursor).await?;
        let response = cursor.into_inner();

        let response = &*response;

        conn.socket.write(response).await?;

        Ok(())
    }
}

/// Get the favicon as a base64 encoded string.
///
/// This is cached in a `OnceCell` to avoid reading the file every time.
async fn get_encoded_favicon() -> &'static String {
    static FAVICON: OnceCell<String> = OnceCell::const_new();
    FAVICON
        .get_or_init(|| async {
            let mut data = Vec::new();
            let Ok(mut image) = tokio::fs::File::open("icon-64.png").await else {
                return String::new();
            };
            image.read_to_end(&mut data).await.unwrap_or_default();
            let data = base64::engine::general_purpose::STANDARD.encode(&data);
            format!("data:image/png;base64,{}", data)
        })
        .await
}
