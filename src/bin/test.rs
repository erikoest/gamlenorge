use std::thread::spawn;
use std::thread::sleep;
use std::time::Duration;
use crossbeam_channel::unbounded;
use crossbeam_channel::{Sender, Receiver};

fn receive(t: i32, rx: Receiver<i32>) {
    let sec = Duration::from_millis(1000);

    while let Ok(i) = rx.recv() {
        println!("Thread {} Received {}", t, i);
        
        sleep(sec);
    }

    println!("Thread {} finishing", t);    
}

fn main() {
   let (tx, rx): (Sender<i32>, Receiver<i32>) = unbounded();

   let rx1 = rx.clone();
   let rec1 = spawn(move || receive(1, rx1));

   let rx2 = rx.clone();
   let rec2 = spawn(move || receive(2, rx2));

   for a in 1..10 {
      tx.send(a).unwrap();
   }

    drop(tx);

    rec1.join().unwrap();
    rec2.join().unwrap();
}
