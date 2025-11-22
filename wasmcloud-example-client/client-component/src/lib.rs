wit_bindgen::generate!({
    world: "client",
    generate_all
});

use exports::wasmcloud::messaging::handler::{Guest, BrokerMessage as HandlerMessage};
use wasmcloud::messaging::consumer::{publish, BrokerMessage as ConsumerMessage};

struct Component;

// Implement the handler interface to receive messages from the remote WebSocket server
impl Guest for Component {
    fn handle_message(msg: HandlerMessage) -> Result<(), String> {
        // This handler receives messages from the remote WebSocket server
        // In a real application, you would:
        // 1. Process the message body
        // 2. Perform business logic
        // 3. Optionally send a response back using the consumer interface
        
        // For this example, we'll echo the message back if it has a reply_to field
        if let Some(reply_to) = msg.reply_to {
            // Send a response back to the remote server
            let response = ConsumerMessage {
                subject: format!("response.{}", msg.subject),
                body: msg.body.clone(),
                reply_to: Some(reply_to),
            };
            
            // Publish response back through the WebSocket provider
            publish(&response).map_err(|e| format!("Failed to send response: {}", e))?;
        }
        
        Ok(())
    }
}

export!(Component);
