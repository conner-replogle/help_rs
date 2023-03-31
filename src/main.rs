use std::{process::{Command, Stdio}, io::{BufReader, BufWriter, Read, Write, BufRead}, thread};

use openai::{set_key, chat::{ChatCompletionBuilder, ChatCompletionMessage, ChatCompletionMessageRole}};


static PROMPT: &'static str ="Your job is to interpret my message and infer the linux command 
I would like to run based off of my message. 
Upon a request you will only respond with the linux command and no explanation or anything else. Your first request is";
use clap::{Parser};

#[derive(Parser)]
#[command(author, version, about="Use Chat-GPT to suggest linux commands", long_about = None)]
struct Cli {
    #[arg(help= "Your request to the bot, leave blank for a conversation")]
    request: Option<String>,
    #[arg(help= "The open ai api key, can also set as env API_KEY",long)]
    api_key: Option<String>,

    #[arg(help= "How many choices the ai should generate",short,default_value_t = 3)]
    num_choices: u8,

    
}



#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    let args = Cli::parse();
    let api_key: String;
    if let Some(key) = args.api_key{
        api_key = key;
    }else{
        api_key = std::env::var("API_KEY").expect("NO API_KEY")
    }
    set_key(api_key);


    if let Some(request) = args.request{
        let chat = ChatCompletionBuilder::default().messages(vec![ChatCompletionMessage{ 
            role: ChatCompletionMessageRole::System, 
            content: format!("{} {}", PROMPT,request).into()
        ,   name: None }])
        .model("gpt-3.5-turbo-0301").user("user23232").n(args.num_choices).create().await.unwrap().unwrap();
        let mut choices:Vec<&openai::chat::ChatCompletionChoice> = chat.choices.iter().take(args.num_choices.into()).collect();
        
        choices.dedup_by(|a,b| a.message.content.eq(&b.message.content));
        for choice in choices{
            println!(
                "Command: {}",
                &choice.message.content.trim()
            );
        }
        return;

        
    }
    let child = Command::new("/bin/bash")
    .stdout(Stdio::piped())
    .stdin(Stdio::piped())
    .spawn()
    .expect("Failed to start bash process");

    let mut stdout = BufReader::new(child.stdout.unwrap());

    let mut stdin = BufWriter::new(child.stdin.unwrap());
    loop {
        println!("What do you need help with? ");
        let mut input  = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let chat = ChatCompletionBuilder::default().messages(vec![ChatCompletionMessage{ 
            role: ChatCompletionMessageRole::System, 
            content: format!("{} {}", PROMPT,input).into()
        ,   name: None }])
        .model("gpt-3.5-turbo-0301").user("user23232").create().await.unwrap().unwrap();

        let returned_message = chat.choices.first().unwrap().message.clone();

        println!(
            "Run Command: {} (y/n/s)",
            &returned_message.content.trim()
        );
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        println!("{line}");
        if line.contains("y"){
            println!("Running {}",returned_message.content.trim());
            let command = format!("{} && echo 23928392\n",returned_message.content.trim());
            
            stdin.write(&command.as_bytes()).unwrap();
            stdin.flush().unwrap();
            loop{
                let mut buf = String::new();
                stdout.read_line(&mut buf).unwrap();
                //let buf = String::from_utf8(buf.to_vec()).unwrap();
                if buf.contains("23928392"){
                    break;
                }else{
                    println!("{buf}");
                }

            }

        }else if line.contains("n"){
            println!("Closed")
        }else{
            return;
        }

    }

    
}
