pub type AsyncMutex<T> = futures_intrusive::sync::Mutex<T>;
pub type AsyncRwLock<T> = async_std::sync::RwLock<T>;
pub type Mutex<T> = parking_lot::Mutex<T>;
pub type RwLock<T> = parking_lot::RwLock<T>;
