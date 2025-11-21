wit_bindgen::generate!({
    world: "echo",
    generate_all
});

use exports::wasmcloud::messaging::handler::{BrokerMessage, Guest};

struct Component;

impl Guest for Component {
    fn handle_message(_msg: BrokerMessage) -> Result<(), String> {
        // This is a simple message handler that acknowledges receipt
        // In a production component, you would:
        // 1. Process the message body
        // 2. Perform business logic
        // 3. Optionally send a response using the messaging consumer interface
        
        // NOTE: This example demonstrates message reception only.
        // To send replies back through the WebSocket, the component would need
        // to import and use wasmcloud:messaging/consumer interface.
        // The provider's session management will route replies to the correct client.
        
        // Example of what a full implementation could look like:
        // if msg.reply_to.is_some() {
        //     use wasmcloud::messaging::consumer;
        //     let response = BrokerMessage {
        //         subject: format!("echo.{}", msg.subject),
        //         body: msg.body, // Echo the body back
        //         reply_to: None,
        //     };
        //     consumer::publish(&response)?;
        // }
        
        // For this basic example, we just acknowledge receipt
        Ok(())
    }
}

export!(Component);
