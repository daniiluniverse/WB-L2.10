// Задача
// Реализовать простейший telnet-клиент.
//
// Примеры вызовов:
//
// telnet --timeout=10s host port
//
// telnet mysite.ru 8080
//
// telnet --timeout=3s 1.1.1.1 123
//
// Требования
// Программа должна подключаться к указанному хосту (ip или доменное имя + порт) по протоколу TCP.
// После подключения STDIN программы должен записываться в сокет, а данные полученные из сокета должны выводиться в STDOUT
// Опционально в программу можно передать таймаут на подключение к серверу (через аргумент --timeout, по умолчанию 10s)
// При нажатии Ctrl+D программа должна закрывать сокет и завершаться. Если сокет закрывается со стороны сервера, программа должна также завершаться.
// При подключении к несуществующему серверу, программа должна завершаться через timeout



use std::io::{self, BufRead, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use std::thread;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    /// Таймаут для подключения
    #[structopt(short = "t", long = "timeout", default_value = "10s")]
    timeout: String,

    /// Хост для подключения
    host: String,

    /// Порт для подключения
    port: u16,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    // Парсинг таймаута
    let timeout = opt.timeout.trim_end_matches('s').parse::<u64>().unwrap_or(10);
    let addr = format!("{}:{}", opt.host, opt.port);

    // Попытка установить соединение с указанным таймаутом
    let socket_addrs = addr.to_socket_addrs()?;
    let addr = match socket_addrs.into_iter().next() {
        Some(addr) => addr,
        None => {
            eprintln!("Не удалось получить адрес для {}", addr);
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Не удалось получить адрес"));
        }
    };

    println!("Попытка подключения к {}...", addr);
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(timeout))?;
    println!("Подключение успешно установлено к {}.", addr);

    // Установка сокета в неблокирующий режим
    let _ = stream.set_nonblocking(true);
    let mut stream_clone = stream.try_clone()?;

    // Поток для чтения из сокета
    thread::spawn(move || {
        let mut buffer = [0; 1024];
        loop {
            match stream_clone.read(&mut buffer) {
                Ok(0) => break, // Соединение закрыто сервером
                Ok(n) => {
                    let data = &buffer[..n];
                    io::stdout().write_all(data).unwrap();
                    io::stdout().flush().unwrap();
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // Будет блокировка, продолжаем
                }
                Err(e) => {
                    eprintln!("Ошибка чтения из сокета: {}", e);
                    break;
                }
            }
        }
    });

    // Основной поток для записи в сокет
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(input) => {
                if input.is_empty() {
                    break; // Выход при пустой строке
                }
                if let Err(e) = stream.write_all(input.as_bytes()) {
                    eprintln!("Ошибка записи в сокет: {}", e);
                    break;
                }
            }
            Err(_) => break, // Завершение работы при ошибке чтения
        }
    }

    // Закрытие сокета
    drop(stream);
    println!("Соединение закрыто. Программа завершена.");
    Ok(())
}
