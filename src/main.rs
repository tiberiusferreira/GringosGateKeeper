extern crate serial;
extern crate flexi_logger;
extern crate teleborg;
extern crate log_panics;
#[macro_use] extern crate log;
#[macro_use] extern crate diesel_infer_schema;
#[macro_use] extern crate diesel;
extern crate failure;

mod database;
mod gatekeeper;
use flexi_logger::{opt_format, Logger};


fn main() {
    log_panics::init();
    Logger::with_str("info")
        .log_to_file()
        .directory("log_files")
        .format(opt_format)
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    let mut gk = gatekeeper::GringosGateKeeperBot::<teleborg::Bot>::new();
    gk.start();




//    port_config(&mut port).unwrap();
//    send_char_to_serial(&mut port).unwrap();
//    read_char_from_serial(&mut port).unwrap();
}



//fn handle_msg(message: Message, db_con: &PgConnection) -> OutgoingMessage{
//    let sender_db_info = message.from.as_ref().and_then(|user| {
//        get_user(db_con, user.id).ok()
//    });
//    let chat_id = message.chat.id;
//    match sender_db_info {
//        Some(user) => {
//            match message.from.as_ref(){
//                Some(sender) => {
//                    let mut message = OutgoingMessage::new(chat_id, "Veja quem está na porta antes de abrir!");
//                    message.with_reply_markup(vec![vec![OPEN.to_string()]]);
//                    message
//                },
//                None => {
//                    error!("Message with no sender");
//                    OutgoingMessage::new(chat_id, "Você não está registrado e não tem sender.id, estranho. Mostre isso para @TiberioFerreira")
//                }
//            }
//        },
//        None => {
//            match message.from.as_ref(){
//                Some(sender) => {
//                    OutgoingMessage::new(chat_id, &format!("Você não está registrado, envie essa mensagem para @TiberioFerreira com seu id: {}", sender.id))
//                },
//                None => {
//                    error!("Message with no sender");
//                    OutgoingMessage::new(chat_id, "Você não está registrado e não tem sender.id, estranho. Mostre isso para @TiberioFerreira")
//                }
//            }
//        }
//    }
//    let message_handler = MessageHandler::new(message, sender_db_info);
//    let response = message_handler.get_response();
//    match response.action {
//        UpdateImpact::AddPicpayAccount {user, picpay_name} => {
//            update_user_picpay(&self.database_connection, user.id, Some(picpay_name));
//        },
//        UpdateImpact::RemovePicpayAccount {user} =>{
//            update_user_picpay(&self.database_connection, user.id, None);
//        },
//        _ => {}
//    }
//    let mut message = OutgoingMessage::new(chat_id, &response.reply);
//    if let Some(markup) = response.reply_markup {
//        message.with_reply_markup(markup);
//    };
//    self.telegram_interface.send_msg(message);



//fn handle_callback(callback_query: CallBackQuery, db_con: &PgConnection) -> OutgoingMessage{
//    let sender_db_info = get_user(db_con, callback_query.from.id);
//    let chat_id = message.chat.id;
//    match sender_db_info {
//        Some(user) => {
//            match callback_query.data{
//                Some(data) => {
//                    match data{
//                        OPEN => {
//                            send_char_to_serial(&mut port).unwrap();
//                            read_char_from_serial(&mut port).unwrap();
//                            let mut message = OutgoingMessage::new(chat_id, "Aberto! Sempre veja quem está na porta antes de abrir!");
//                            message.with_reply_markup(vec![vec![OPEN.to_string()]]);
//                            message
//                        },
//                        _ => {
//                            let mut message = OutgoingMessage::new(chat_id, "Callback diferente do esperado. Estranho... Envie isso para @TiberioFerreira");
//                            message.with_reply_markup(vec![vec![OPEN.to_string()]]);
//                            message
//                        }
//                    }
//                },
//                None => {
//                    let mut message = OutgoingMessage::new(chat_id, "Sem Callback. Estranho... Envie isso para @TiberioFerreira");
//                    message.with_reply_markup(vec![vec![OPEN.to_string()]]);
//                    message
//                }
//            },
//            None => {
//            }
//        }
//    },
//    None => {
//    match message.from.as_ref(){
//    Some(sender) => {
//    OutgoingMessage::new(chat_id, &format!("Você não está registrado, envie essa mensagem para @TiberioFerreira com seu id: {}", sender.id))
//    },
//    None => {
//    error!("Message with no sender");
//    OutgoingMessage::new(chat_id, "Você não está registrado e não tem sender.id, estranho. Mostre isso para @TiberioFerreira")
//    }
//    }
//    }
//
//
//}
//
//pub fn handle_update(update: Update, db_con: &PgConnection){
//    if let Some(message) = update.message {
//        info!("This was a message update");
//        handle_msg(message, db_con);
//        return;
//    }
//    if let Some(callback_query) = update.callback_query {
//        info!("This was a callback update");
////        handle_callback(callback_query, db_con);
//
//        return;
//    }
//    error!("This was neither a Message or Callback update. Weird.");
//}
//
//fn get_updates_list<T: TelegramInterface>(telegram_api: &T) -> Vec<Update>{
//    let updates_channel = telegram_api.get_updates_channel();
//    loop {
//        let possible_updates_list = updates_channel.recv();
//        match possible_updates_list {
//            Ok(update_list) => return update_list,
//            Err(e) => {
//                error!("Error while getting updates list from Teleborg: {}", e);
//            }
//        };
//    }
//}

//fn port_config<T: SerialPort>(port: &mut T) -> serial::Result<()> {
//    port.reconfigure(&|settings| {
//        settings.set_baud_rate(serial::Baud9600)?;
//        settings.set_char_size(serial::Bits8);
//        settings.set_parity(serial::ParityNone);
//        settings.set_stop_bits(serial::Stop1);
//        settings.set_flow_control(serial::FlowNone);
//        Ok(())
//    })
//}
//

//fn read_char_from_serial<T: SerialPort>(port: &mut T) -> serial::Result<char>{
//    port.set_timeout(Duration::from_millis(5000))?;
//
//    let mut buffer :[u8 ; 1] = [0; 1];
//// read up to 10 bytes
//    port.read(&mut buffer)?;
//
////    println!("{:?}", buf2);
//    println!("Read: {:?} bytes", buffer);
//    let char_u8 = buffer.get(0).unwrap();
//    let a = char_u8.clone() as char;
//    println!("Read char: {:?}", a);
//    Ok(a)
//}
//
//fn send_char_to_serial<T: SerialPort>(port: &mut T) -> serial::Result<()>{
//    port.set_timeout(Duration::from_millis(1000))?;
//
//    let mut buffer :[u8 ; 1] = [ 'O' as u8; 1];
//// read up to 10 bytes
//    port.write(&mut buffer)?;
//
////    println!("{:?}", buf2);
//    Ok(())
//}