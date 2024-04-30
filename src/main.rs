use std::sync::Arc;

use tokens::token_store::TokenStore;

mod tokens;

use clap::{Parser, Subcommand};

use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use prettytable::{row, Cell, Row, Table};

#[derive(Parser)]
#[command(name = "mellon")]
#[command(bin_name = "mellon")]
#[command(version = "0.0.1")]
#[command(author = "Daniel du Plessis")]
#[command(about = "A small, simple, fast auth service")]
#[command(long_about = THE_DOORS_OF_DURIN)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Starts the auth server.
    Serve {
        /// Hostname for the server.
        #[clap(
            value_name = "HOSTNAME",
            required = false,
            default_value = "localhost:8090"
        )]
        host: Option<String>,
    },

    /// Manage tokens by adding or removing.
    Token {
        #[clap(subcommand)]
        action: TokenCommands,
    },
}

#[derive(Debug, Subcommand)]
enum TokenCommands {
    /// Add a new token.
    Add {
        /// The label of the token to add
        token_label: String,
    },

    /// Revoke an existing token by its label.
    Rescind {
        /// The label of the token to remove.
        token_label: String,
    },

    /// List all tokens previously issued
    List {},
}

fn main() {
    let token_store = match TokenStore::new(STORE_FILE_PATH.to_string()) {
        Ok(store) => store,
        Err(_) => {
            println!("Failed to instantiate token store");
            return;
        }
    };
    let args = Cli::parse();
    match args.command {
        Commands::Serve { host } => match host {
            Some(host) => {
                println!("Server starting up on {}", host);
                match serve(host.as_str(), token_store) {
                    Ok(_) => println!("Server shut down!"),
                    Err(err) => println!("Failed to host server: {}", err),
                }
            }
            None => println!("Host is not defined properly!"),
        },
        Commands::Token { action } => match action {
            TokenCommands::Add { token_label } => add_token(token_store, token_label),
            TokenCommands::Rescind { token_label } => rescind_token(token_store, token_label),
            TokenCommands::List {} => list_tokens(token_store),
        },
    }
}

#[get("/auth")]
async fn auth(req: HttpRequest, token_store: web::Data<Arc<TokenStore>>) -> impl Responder {
    let auth_header = req.headers().get("Authorization");
    if let Some(header_value) = auth_header {
        if let Ok(str_value) = header_value.to_str() {
            if let Some(token) = str_value.strip_prefix("Bearer ") {
                if token_store.contains_token(token).unwrap() {
                    return HttpResponse::Ok().body("Authorized");
                } else {
                    println!("Refused access to token {}", token);
                }
            }
        }
    }

    HttpResponse::Unauthorized().body("Unauthorized")
}

#[actix_web::main]
async fn serve(host: &str, token_store: TokenStore) -> std::io::Result<()> {
    let token_store = Arc::new(token_store);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(token_store.clone()))
            .service(auth)
    })
    .bind(host)?
    .run();

    server.await
}

fn rescind_token(mut token_store: TokenStore, label: String) {
    let result = token_store.rescind(label.as_str());
    match result {
        Ok(_) => println!(
            "Token with label {} has been removed. Be sure to restart server to load changes!",
            label
        ),
        Err(err) => println!("Failed to rescind token: {}", err),
    }
}

fn add_token(mut token_store: TokenStore, label: String) {
    let new_token = token_store.create(label.as_str());
    let new_token = match new_token {
        Ok(uuid) => uuid,
        Err(error) => {
            println!("Failed to generate new token for label: {}", error);
            return;
        }
    };
    println!("{}", new_token.1);
}

fn list_tokens(token_store: TokenStore) {
    match token_store.iter() {
        Ok(iter) => {
            let mut table = Table::new();
            table.add_row(row!["Label", "Token"]);
            for token in iter {
                table.add_row(Row::new(vec![
                    Cell::new(token.0.as_str()),
                    Cell::new(
                        ("*".repeat(token.1.len().saturating_sub(4))
                            + &token.1[token.1.len().saturating_sub(4)..])
                            .as_str(),
                    ),
                ]));
            }
            table.printstd();
        }
        Err(err) => println!("Unable to list tokens: {}", err),
    }
}

const STORE_FILE_PATH: &str = "/tmp/token_test";

const THE_DOORS_OF_DURIN: &str = r#"

             _,-'_,-----------._`-._    
           ,'_,-'  ___________  `-._`.
         ,','  _,-'___________`-._  `.`.
       ,','  ,'_,-'     .     `-._`.  `.`.
      /,'  ,','        >|<        `.`.  `.\
     //  ,','      ><  ,^.  ><      `.`.  \\
    //  /,'      ><   / | \   ><      `.\  \\
   //  //      ><    \/\^/\/    ><      \\  \\
  ;;  ;;              `---'              ::  ::
  ||  ||              (____              ||  ||
 _||__||_            ,'----.            _||__||_
(o.____.o)____        `---'        ____(o.____.o)
  |    | /,--.)                   (,--.\ |    |
  |    |((  -`___               ___`   ))|    |
  |    | \\,'',  `.           .'  .``.// |    |
  |    |  // (___,'.         .'.___) \\  |    |
 /|    | ;;))  ____) .     . (____  ((\\ |    |\
 \|.__ | ||/ .'.--.\/       `/,--.`. \;: | __,|;
  |`-,`;.| :/ /,'  `)-'   `-('  `.\ \: |.;',-'|
  |   `..  ' / \__.'         `.__/ \ `  ,.'   |
  |    |,\  /,                     ,\  /,|    |
  |    ||: : )          .          ( : :||    |
 /|    |:; |/  .      ./|\,      ,  \| :;|    |\
 \|.__ |/  :  ,/-    <--:-->    ,\.  ;  \| __,|;
  |`-.``:   `'/-.     '\|/`     ,-\`;   ;'',-'|
  |   `..   ,' `'       '       `  `.   ,.'   |
  |    ||  :                         :  ||    |
  |    ||  |                         |  ||    |
  |    ||  |                         |  ||    |
  |    |'  |            _            |  `|    |
  |    |   |          '|))           |   |    |
  ;____:   `._        `'           _,'   ;____:
 {______}     \___________________/     {______}
 |______|_______________________________|______|
                                          
          _ _/   _  _ //     _  _  '      
       /)(-(/() //)(-((()/) (/ //)//)/)() 
      /                                   
      
      A small, friendly, fast, auth serer.
"#;
