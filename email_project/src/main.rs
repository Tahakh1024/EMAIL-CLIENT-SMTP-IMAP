use std::fs; // Import the standard library's file system module for file operations.
use std::io; // Import the standard library's input/output module for user interaction.
use lettre::transport::smtp::authentication::Credentials; // Import Lettre's Credentials for SMTP authentication.
use lettre::{Message, SmtpTransport, Transport}; // Import Lettre's components for sending emails via SMTP.
use imap::error::Result; // Import the Result type for handling IMAP errors.
use native_tls::TlsConnector; // Import TlsConnector from native_tls for secure connections.

#[derive(serde::Serialize, serde::Deserialize)] // Define a custom struct that can be serialized and deserialized from JSON.
struct EmailData {
    recipient: String, // Email recipient's address.
    subject: String,   // Email subject.
    body: String,      // Email body content.
}

fn send_email(all_email_data: &mut Vec<EmailData>) {
    // Define SMTP configuration.
    let smtp_key = "xyz";
    let from_email = "xyz@gmail.com";
    let host = "smtp-relay.sendinblue.com";

    // Prompt the user for recipient email address.
    println!("Enter recipient email:");
    let mut to_email = String::new();
    io::stdin().read_line(&mut to_email).unwrap();
    to_email = to_email.trim().to_string();

    // Prompt the user for email subject.
    println!("Enter subject:");
    let mut subject = String::new();
    io::stdin().read_line(&mut subject).unwrap();
    subject = subject.trim().to_string();

    // Prompt the user for email body.
    println!("Enter body:");
    let mut body = String::new();
    io::stdin().read_line(&mut body).unwrap();
    body = body.trim().to_string();

    // Build an email message.
    let email = Message::builder()
        .from(from_email.parse().unwrap())
        .to(to_email.parse().unwrap())
        .subject(&subject)
        .body(body.clone())
        .unwrap();

    // Create credentials for SMTP authentication.
    let creds = Credentials::new(from_email.to_string(), smtp_key.to_string());

    // Set up the SMTP mailer and send the email.
    let mailer = SmtpTransport::relay(&host)
        .unwrap()
        .credentials(creds)
        .build();
    mailer.send(&email).unwrap();

    // Create an EmailData struct to store the sent email.
    let email_data = EmailData {
        recipient: to_email.clone(),
        subject: subject.clone(),
        body: body.clone(),
    };

    // Add the sent email data to the all_email_data vector.
    all_email_data.push(email_data);

    // Serialize and write all the email data to a file.
    let json = serde_json::to_string(all_email_data).unwrap();
    fs::write("emails.json", json).unwrap();
}

fn display_sent_emails(all_email_data: &Vec<EmailData>) {
    // Loop through sent emails and display their details.
    for (index, email) in all_email_data.iter().enumerate() {
        println!("Email #{}", index + 1);
        println!("Recipient: {}", email.recipient);
        println!("Subject: {}", email.subject);
        println!("Body: {}", email.body);
        println!("----------------------------");
    }
}

fn display_inbox_emails() {
    // Call the fetch_inbox_emails function to retrieve and display inbox emails.
    match fetch_inbox_emails() {
        Ok(emails) => {
            if emails.is_empty() {
                println!("No emails found in the INBOX.");
            } else {
                println!("List of emails:\n{}", emails.join("\n"));
            }
        }
        Err(err) => {
            eprintln!("Error: {:?}", err);
        }
    }
}

fn fetch_inbox_emails() -> Result<Vec<String>> {
    let domain = "imap.gmail.com";
    let tls = TlsConnector::builder().build().unwrap();

    // Create an IMAP client and log in with your credentials.
    let client = imap::connect((domain, 993), domain, &tls).unwrap();
    let mut imap_session = client
        .login("xyz@gmail.com", "xyzw")
        .map_err(|e| e.0)?;

    // Select the INBOX mailbox.
    imap_session.select("INBOX")?;

    // Fetch the list of email messages (headers only).
    let messages = imap_session.fetch("5:1", "ENVELOPE")?;
    let email_list: Vec<String> = messages.iter().filter_map(|m| {
            // Extract subject and from fields from email headers.
            if let Some(envelope) = m.envelope() {
                let subject = envelope.subject.as_ref().map(|s| String::from_utf8_lossy(s).to_string()).unwrap_or("".to_string());
                let from = envelope.from.as_ref().and_then(|f| f[0].mailbox.as_ref()).map(|m| String::from_utf8_lossy(m).to_string()).unwrap_or("".to_string());
                let date = envelope.date.as_ref().map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or("".to_string());

                Some(format!("\nFrom: {}\nSubject: {}\nDate: {}", from, subject, date))
            } else {
                None
            }
        })
        .collect();

    // Be nice to the server and log out.
    imap_session.logout()?;

    // Return the list of email information.
    Ok(email_list)
}

fn main() {
    // Initialize the all_email_data vector by reading saved email data from a file.
    let mut all_email_data: Vec<EmailData> = match fs::read_to_string("emails.json") {
        Ok(content) => serde_json::from_str(&content).unwrap(),
        Err(_) => Vec::new(),
    };

    // Display a welcome message to the user.
    println!("Welcome to the Email Application!");

    // Main menu loop for user interaction.
    loop {
        println!("Choose an option:");
        println!("1. Send an email");
        println!("2. Display sent emails");
        println!("3. Display inbox emails");
        println!("4. Quit");

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => send_email(&mut all_email_data), // Call the send_email function.
            "2" => display_sent_emails(&all_email_data), // Call the display_sent_emails function.
            "3" => display_inbox_emails(), // Call the display_inbox_emails function.
            "4" => break, // Exit the program.
            _ => println!("Invalid option. Please select 1, 2, 3, or 4."),
        }
    }
}