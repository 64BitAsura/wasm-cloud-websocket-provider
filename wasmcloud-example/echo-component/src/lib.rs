wit_bindgen::generate!({
    world: "echo",
    generate_all
});

use exports::wasmcloud::messaging::handler::{BrokerMessage, Guest};

struct Component;

impl Guest for Component {
    fn handle_message(msg: BrokerMessage) -> Result<(), String> {
        // Log the received message (in a real component, you'd use proper logging)
        // This is a simple echo handler that acknowledges message receipt
        
        // If there's a reply-to field, we could send a response back using
        // the consumer interface, but for this simple example, we just
        // acknowledge receipt
        if msg.reply_to.is_some() {
            // In a full implementation:
            // let _response = BrokerMessage {
            //     subject: format!("echo.response.{}", msg.subject),
            //     body: msg.body.clone(),
            //     reply_to: None,
            // };
            // wasmcloud::messaging::consumer::publish(&_response)?;
        }
        
        Ok(())
    }
}

export!(Component);
