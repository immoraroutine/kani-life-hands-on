use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use dotenvy::dotenv;
use reqwest::Client;
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    enable_raw_mode()?;

    let client = Client::new();
    let url = env::var("URL").expect("URL is not set");
    let mut command = Command::Spawn {
        name: env::var("USER_NAME").expect("user_name is not set"),
        hue: env::var("USER_HUE")
            .expect("hue is not set")
            .parse()
            .expect("hue is not a number"),
    };
    let token = request_command(&url, &client, &command).await?;

    loop {
        // キー入力を待つ
        if event::poll(Duration::from_millis(1000))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    // 左矢印キー
                    KeyCode::Left => {
                        command = Command::Walk {
                            token: token.clone(),
                            side: Side::Left,
                        };
                    }
                    // 右矢印キー
                    KeyCode::Right => {
                        command = Command::Walk {
                            token: token.clone(),
                            side: Side::Right,
                        };
                    }
                    // 左を向く
                    KeyCode::Char('l') => {
                        command = Command::Turn {
                            token: token.clone(),
                            side: Side::Left,
                        };
                    }
                    // 右を向く
                    KeyCode::Char('r') => {
                        command = Command::Turn {
                            token: token.clone(),
                            side: Side::Right,
                        };
                    }
                    // 終了キー (例: 'q')
                    KeyCode::Char('q') => {
                        break;
                    }
                    _ => {
                        command = Command::Ping;
                    }
                }
                request_command(&url, &client, &command).await?;
            }
        }
    }

    // 終了時にターミナルの設定を元に戻す
    disable_raw_mode()?;

    Ok(())
}

async fn request_command(
    url: &str,
    client: &Client,
    command: &Command,
) -> Result<String, Box<dyn std::error::Error>> {
    let result = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&command)
        .send()
        .await?
        .json::<CommandResult>()
        .await?;

    match result {
        CommandResult::Spawn { token } => Ok(token),
        _ => Ok("リクエストに成功".to_string()),
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "type")]
enum Command {
    Ping,
    Spawn { name: String, hue: f32 },
    // Scan {
    //     token: String,
    // },
    Turn { token: String, side: Side },
    Walk { token: String, side: Side },
}

#[derive(Debug, serde::Serialize)]
enum Side {
    Right,
    Left,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type")]
enum CommandResult {
    Pong,
    Spawn {
        token: String,
    },
    Turn,
    Walk {
        // success: bool,
        // point: u16,
        // total_point: u16,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_encoding() {
        let expected = r#"{"type":"Ping"}"#;
        let actual = serde_json::to_string(&Command::Ping).unwrap();
        assert_eq!(expected, actual);
    }
}
