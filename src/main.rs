use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use reqwest::Client;
use std::io;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

    let client = Client::new();
    let url = "https://kani-life-m6xtjae7da-an.a.run.app/api/command";
    let mut command = Command::Spawn {
        name: "koko".to_string(),
        hue: 300.0,
    };
    let token = request_server(url, &client, &command).await?;

    loop {
        // キー入力を待つ
        if event::poll(Duration::from_nanos(1))? {
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
                request_server(url, &client, &command).await?;
            }
        }
    }

    // 終了時にターミナルの設定を元に戻す
    disable_raw_mode()?;
    execute!(stdout, crossterm::terminal::LeaveAlternateScreen)?;

    Ok(())
}

async fn request_server(
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
        success: bool,
        point: u16,
        total_point: u16,
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
