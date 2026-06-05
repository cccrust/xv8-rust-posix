use xv8_user_std::time::Duration;

pub async fn sleep(duration: Duration) {
    xv8_async::sleep(duration).await
}
