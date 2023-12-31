use crate::config::settings::Settings;
use lettre::message::MultiPart;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rocket::yansi::Paint;

pub struct NotificationManager {
    pub smtp: SmtpTransport,
    pub settings: Settings,
}

impl NotificationManager {
    pub fn new(settings: Settings) -> Self {
        let credentials = Credentials::new(
            settings.email.user.to_string(),
            settings.email.password.to_string(),
        );

        let smtp = SmtpTransport::starttls_relay(settings.email.host.as_str())
            .unwrap()
            .port(settings.email.port.parse::<u16>().unwrap())
            .credentials(credentials)
            .build();

        let test = smtp.test_connection();

        if test.is_err() {
            println!(
                "{} {}: {}",
                Paint::red("Error:"),
                Paint::bold("SMTP Connection Failed"),
                test.err().unwrap()
            );
        } else {
            println!("{} Connected to SMTP Server", Paint::green("Success:"));
        }

        NotificationManager { smtp, settings }
    }

    pub fn _send_notification(&self, to: String, subject: String, body: MultiPart) {
        let message = Message::builder()
            .from(self.settings.email.from.as_str().parse().unwrap())
            .reply_to(self.settings.email.reply_to.as_str().parse().unwrap())
            .to(to.parse().unwrap())
            .subject(subject)
            .multipart(body)
            .unwrap();

        self.smtp.send(&message).unwrap();
    }
}
