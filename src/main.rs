use std::fmt::{Debug, Display};

use tokio::task::JoinError;

use rust_graphql_template::startup::{App, Telemetry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = Telemetry::get_subscriber("rust_graphql_template", "info");
    Telemetry::init_subscriber(subscriber);
    let application = App::new().await?;
    let application_task = tokio::spawn(application.run_until_stopped());

    tokio::select! {
        o = application_task => report_exit("API", o),
    };

    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{}' task failed to complete",
                task_name
            )
        }
    }
}
