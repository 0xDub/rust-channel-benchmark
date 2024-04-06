use rand::{RngCore, thread_rng};
use postage::prelude::Stream;
use postage::prelude::Sink;
use std::sync::Arc;
use magnetic::{Producer, Consumer};

mod utils;
use utils::{CL, FileHandler};


// =-= Experiments =-=-= //

#[derive(Debug)]
pub enum AsyncExperiments {
    Kanal,
    Flume,
    Tokio,
    AsyncChannel,
    Crossfire,
    Thingbuf,
    Tachyonix,
    Postage,
    AsyncBroadcast,
}

#[derive(Debug, Clone)]
pub enum InterProcessSync {
    Rtrb,
    Ringbuf,
    CrossbeamChannel,
    CrossbeamQueue,
    Kanal,
    Flume,
    Omango,
    Npnc,
    Magnetic,
    Tokio
}

// =-= Data being passed =-=-= //

#[derive(Debug, Clone)]
pub struct Data {
    pub time: minstant::Instant,
    pub data: Vec<u8>,
    pub kill: bool,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            time: minstant::Instant::now(),
            data: vec![],
            kill: false,
        }
    }
}

impl Data {
    pub fn new(data: Vec<u8>, kill: bool) -> Self {
        Data {
            time: minstant::Instant::now(),
            data,
            kill,
        }
    }
}

// =-= Inter-Process =-=-= //
// - Cross CPUs / Threads communication
// - Asynchronous vs Synchronous (blocking + spinning)

fn inter_process_async(sample_size: i64, payload_size_in_bytes: i64, cpu_0: usize, cpu_1: usize) {
    CL::Teal.print("[+][Inter-Process][Async] Starting");

    let mut kanal_sender_handler =            FileHandler::new("inter_latencies/async/kanal_sender.txt").unwrap();
    let mut flume_sender_handler =            FileHandler::new("inter_latencies/async/flume_sender.txt").unwrap();
    let mut tokio_sender_handler =            FileHandler::new("inter_latencies/async/tokio_sender.txt").unwrap();
    let mut async_channel_sender_handler =    FileHandler::new("inter_latencies/async/async_channel_sender.txt").unwrap();
    let mut crossfire_sender_handler =        FileHandler::new("inter_latencies/async/crossfire_sender.txt").unwrap();
    let mut thingbuf_sender_handler =         FileHandler::new("inter_latencies/async/thingbuf_sender.txt").unwrap();
    let mut tachyonix_sender_handler =        FileHandler::new("inter_latencies/async/tachyonix_sender.txt").unwrap();
    let mut postage_sender_handler =          FileHandler::new("inter_latencies/async/postage_sender.txt").unwrap();
    let mut async_broadcast_sender_handler =  FileHandler::new("inter_latencies/async/async_broadcast_sender.txt").unwrap();

    let (kanal_tx, kanal_rx) = kanal::bounded_async::<Data>(128);
    let (flume_tx, flume_rx) = flume::bounded::<Data>(128);
    let (tokio_tx, mut tokio_rx) = tokio::sync::mpsc::channel::<Data>(128);
    let (async_channel_tx, async_channel_rx) = async_channel::bounded::<Data>(128);
    let (crossfire_tx, crossfire_rx) = crossfire::mpsc::bounded_future_both::<Data>(128);
    let (thingbuf_tx, thingbuf_rx) = thingbuf::mpsc::channel::<Data>(128);
    let (tachyonix_tx, mut tachyonix_rx) = tachyonix::channel::<Data>(128);
    let (mut postage_tx, mut postage_rx) = postage::mpsc::channel::<Data>(128);
    let (async_broadcast_tx, mut async_broadcast_rx) = async_broadcast::broadcast::<Data>(128);


    let mut rng = thread_rng();
    let mut random_data = vec![0; payload_size_in_bytes as usize];
    rng.fill_bytes(&mut random_data);


    let sender_handle = std::thread::spawn(move || { // create a system thread
        let res = core_affinity::set_for_current(core_affinity::CoreId { id: cpu_0 }); // pin thread to specific core
        if res {

            // create tokio runtime
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .worker_threads(1)
                .build()
                .expect(&CL::Red.str("[!][Inter-Process][Async] Failed to create runtime"));
            let local = tokio::task::LocalSet::new();
            local.block_on(&runtime, async {

                let experiments = vec![
                    AsyncExperiments::Kanal,
                    AsyncExperiments::Flume,
                    AsyncExperiments::Tokio,
                    AsyncExperiments::AsyncChannel,
                    AsyncExperiments::Crossfire,
                    AsyncExperiments::Thingbuf,
                    AsyncExperiments::Tachyonix,
                    AsyncExperiments::Postage,
                    AsyncExperiments::AsyncBroadcast,
                ];

                for experiment in experiments {
                    CL::Dull.print_literal(format!("[-] {:?} | Running experiment...", experiment));
                    match experiment {
                        AsyncExperiments::Kanal => {
                            for _ in 0..sample_size {

                                let send_time = minstant::Instant::now();
                                kanal_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Inter-Process][Async] Kanal | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                kanal_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Async] Kanal | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            kanal_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Inter-Process][Async] Kanal | Failed to send message"));
                        },
                        AsyncExperiments::Flume => {
                            for _ in 0..sample_size {

                                let send_time = minstant::Instant::now();
                                flume_tx.send_async(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Inter-Process][Async] Flume | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                flume_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Async] Flume | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            flume_tx.send_async(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Inter-Process][Async] Flume | Failed to send message"));
                        },
                        AsyncExperiments::Tokio => {
                            for _ in 0..sample_size {
                                let send_time = minstant::Instant::now();
                                tokio_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Inter-Process][Async] Tokio | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                tokio_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Async] Tokio | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            tokio_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Inter-Process][Async] Tokio | Failed to send message"));
                        },
                        AsyncExperiments::AsyncChannel => {
                            for _ in 0..sample_size {

                                let send_time = minstant::Instant::now();
                                async_channel_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Inter-Process][Async] Async Channel | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                async_channel_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Async] Async Channel | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            async_channel_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Inter-Process][Async] Async Channel | Failed to send message"));
                        },
                        AsyncExperiments::Crossfire => {
                            for _ in 0..sample_size {

                                let send_time = minstant::Instant::now();
                                crossfire_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Inter-Process][Async] Crossfire | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                crossfire_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Async] Crossfire | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            crossfire_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Inter-Process][Async] Crossfire | Failed to send message"));
                        },
                        AsyncExperiments::Thingbuf => {
                            for _ in 0..sample_size {

                                let send_time = minstant::Instant::now();
                                thingbuf_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Inter-Process][Async] Thingbuf | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                thingbuf_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Async] Thingbuf | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            thingbuf_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Inter-Process][Async] Thingbuf | Failed to send message"));
                        },
                        AsyncExperiments::Tachyonix => {
                            for _ in 0..sample_size {

                                let send_time = minstant::Instant::now();
                                tachyonix_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Inter-Process][Async] Tachyonix | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                tachyonix_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Async] Tachyonix | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            tachyonix_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Inter-Process][Async] Tachyonix | Failed to send message"));
                        },
                        AsyncExperiments::Postage => {
                            for _ in 0..sample_size {

                                let send_time = minstant::Instant::now();
                                postage_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Inter-Process][Async] Postage | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                postage_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Async] Postage | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            postage_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Inter-Process][Async] Postage | Failed to send message"));
                        },
                        AsyncExperiments::AsyncBroadcast => {
                            for _ in 0..sample_size {

                                let send_time = minstant::Instant::now();
                                async_broadcast_tx.broadcast(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Inter-Process][Async] Async Broadcast | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                async_broadcast_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Async] Async Broadcast | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            async_broadcast_tx.broadcast(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Inter-Process][Async] Async Broadcast | Failed to send message"));
                        },
                    }
                }

            });
        }
    });

    let mut kanal_receiver_handler =            FileHandler::new("inter_latencies/async/kanal_receiver.txt").unwrap();
    let mut flume_receiver_handler =            FileHandler::new("inter_latencies/async/flume_receiver.txt").unwrap();
    let mut tokio_receiver_handler =            FileHandler::new("inter_latencies/async/tokio_receiver.txt").unwrap();
    let mut async_channel_receiver_handler =    FileHandler::new("inter_latencies/async/async_channel_receiver.txt").unwrap();
    let mut crossfire_receiver_handler =        FileHandler::new("inter_latencies/async/crossfire_receiver.txt").unwrap();
    let mut thingbuf_receiver_handler =         FileHandler::new("inter_latencies/async/thingbuf_receiver.txt").unwrap();
    let mut tachyonix_receiver_handler =        FileHandler::new("inter_latencies/async/tachyonix_receiver.txt").unwrap();
    let mut postage_receiver_handler =          FileHandler::new("inter_latencies/async/postage_receiver.txt").unwrap();
    let mut async_broadcast_receiver_handler =  FileHandler::new("inter_latencies/async/async_broadcast_receiver.txt").unwrap();

    let receiver_handle = std::thread::spawn(move || { // create a system thread
        let res = core_affinity::set_for_current(core_affinity::CoreId { id: cpu_1 }); // pin thread to specific core
        if res {

            // create tokio runtime
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .worker_threads(1)
                .build()
                .expect(&CL::Red.str("[!][Inter-Process][Async] Failed to create runtime"));
            let local = tokio::task::LocalSet::new();
            local.block_on(&runtime, async {

                let mut handles = Vec::new();

                let kanal_handler = tokio::task::spawn_local(async move {
                    loop {
                        match kanal_rx.recv().await {
                            Ok(data) => {
                                if data.kill { break }

                                let latency = data.time.elapsed().as_nanos();
                                kanal_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Async] Kanal | Failed to write to file"));

                            },
                            Err(e) => {
                                CL::Red.print(&format!("[!][Inter-Process][Async] Kanal | Failed to receive message: {}", e));
                                break;
                            }
                        }
                    }
                });
                handles.push(kanal_handler);

                let flume_handler = tokio::task::spawn_local(async move {
                    loop {
                        match flume_rx.recv_async().await {
                            Ok(data) => {
                                if data.kill { break }

                                let latency = data.time.elapsed().as_nanos();
                                flume_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Async] Flume | Failed to write to file"));

                            },
                            Err(e) => {
                                CL::Red.print(&format!("[!][Inter-Process][Async] Flume | Failed to receive message: {}", e));
                                break;
                            }
                        }
                    }
                });
                handles.push(flume_handler);

                let tokio_handler = tokio::task::spawn_local(async move {
                    while let Some(data) = tokio_rx.recv().await {
                        if data.kill { break }

                        let latency = data.time.elapsed().as_nanos();
                        tokio_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Async] Tokio | Failed to write to file"));
                    }
                });
                handles.push(tokio_handler);

                let async_channel_handler = tokio::task::spawn_local(async move {
                    loop {
                        match async_channel_rx.recv().await {
                            Ok(data) => {
                                if data.kill { break }

                                let latency = data.time.elapsed().as_nanos();
                                async_channel_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Async] Async Channel | Failed to write to file"));

                            },
                            Err(e) => {
                                CL::Red.print(&format!("[!][Inter-Process][Async] Async Channel | Failed to receive message: {}", e));
                                break;
                            }
                        }
                    }
                });
                handles.push(async_channel_handler);

                let crossfire_handler = tokio::task::spawn_local(async move {
                    loop {
                        match crossfire_rx.recv().await {
                            Ok(data) => {
                                if data.kill { break }

                                let latency = data.time.elapsed().as_nanos();
                                crossfire_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Async] Crossfire | Failed to write to file"));

                            },
                            Err(e) => {
                                CL::Red.print(&format!("[!][Inter-Process][Async] Crossfire | Failed to receive message: {}", e));
                                break;
                            }
                        }
                    }
                });
                handles.push(crossfire_handler);

                let thingbuf_handler = tokio::task::spawn_local(async move {
                    loop {
                        match thingbuf_rx.recv().await {
                            Some(data) => {
                                if data.kill { break }

                                let latency = data.time.elapsed().as_nanos();
                                thingbuf_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Async] Thingbuf | Failed to write to file"));

                            },
                            None => {
                                CL::Red.print(&format!("[!][Inter-Process][Async] Thingbuf | Failed to receive message"));
                                break;
                            }
                        }
                    }
                });
                handles.push(thingbuf_handler);

                let tachyonix_handler = tokio::task::spawn_local(async move {
                    loop {
                        match tachyonix_rx.recv().await {
                            Ok(data) => {
                                if data.kill { break }

                                let latency = data.time.elapsed().as_nanos();
                                tachyonix_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Async] Tachyonix | Failed to write to file"));

                            },
                            Err(e) => {
                                CL::Red.print(&format!("[!][Inter-Process][Async] Tachyonix | Failed to receive message: {}", e));
                                break;
                            }
                        }
                    }
                });
                handles.push(tachyonix_handler);

                let postage_handler = tokio::task::spawn_local(async move {
                    loop {
                        match postage_rx.recv().await {
                            Some(data) => {
                                if data.kill { break }

                                let latency = data.time.elapsed().as_nanos();
                                postage_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Async] Postage | Failed to write to file"));

                            },
                            None => {
                                CL::Red.print(&format!("[!][Inter-Process][Async] Postage | Failed to receive message"));
                                break;
                            }
                        }
                    }
                });
                handles.push(postage_handler);

                let async_broadcast_handler = tokio::task::spawn_local(async move {
                    loop {
                        match async_broadcast_rx.recv().await {
                            Ok(data) => {
                                if data.kill { break }

                                let latency = data.time.elapsed().as_nanos();
                                async_broadcast_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Async] Async Broadcast | Failed to write to file"));

                            },
                            Err(e) => {
                                CL::Red.print(&format!("[!][Inter-Process][Async] Async Broadcast | Failed to receive message: {}", e));
                                break;
                            }
                        }
                    }
                });
                handles.push(async_broadcast_handler);

                for handle in handles {
                    handle.await.expect(&CL::Red.str("[!][Inter-Process][Async] Receiver_handle | Failed to await a handle"));
                }

            });
        }
    });

    receiver_handle.join().unwrap();
    sender_handle.join().unwrap();

    CL::DullGreen.print("[+][Inter-Process][Async] Complete\n");

}

fn inter_process_busy_spin(sample_size: i64, payload_size_in_bytes: i64, cpu_0: usize, cpu_1: usize) {
    CL::Teal.print("[+][Inter-Process][Busy] Starting");

    // this setup will have to be a bit different as one of the cores will be busy spinning and consuming all the resources.
    // so, we'll iter through the impls and create a new thread / core for each one of them.
    // afterwards, business as usual!

    let experiments = vec![
        InterProcessSync::Rtrb,
        InterProcessSync::Ringbuf,
        InterProcessSync::CrossbeamChannel,
        InterProcessSync::CrossbeamQueue,
        InterProcessSync::Kanal,
        InterProcessSync::Flume,
        InterProcessSync::Omango,
        InterProcessSync::Npnc,
        InterProcessSync::Magnetic,
        InterProcessSync::Tokio,
    ];
    

    for experiment in experiments {
        CL::Dull.print_literal(format!("[-] {:?} | Running experiment...", experiment));
        
        let mut rtrb_sender_handler =               FileHandler::new("inter_latencies/busy/rtrb_sender.txt").unwrap();
        let mut ringbuf_sender_handler =            FileHandler::new("inter_latencies/busy/ringbuf_sender.txt").unwrap();
        let mut crossbeam_channel_sender_handler =  FileHandler::new("inter_latencies/busy/crossbeam_channel_sender.txt").unwrap();
        let mut crossbeam_queue_sender_handler =    FileHandler::new("inter_latencies/busy/crossbeam_queue_sender.txt").unwrap();
        let mut kanal_sender_handler =              FileHandler::new("inter_latencies/busy/kanal_sender.txt").unwrap();
        let mut flume_sender_handler =              FileHandler::new("inter_latencies/busy/flume_sender.txt").unwrap();
        let mut omango_sender_handler =             FileHandler::new("inter_latencies/busy/omango_sender.txt").unwrap();
        let mut npnc_sender_handler =               FileHandler::new("inter_latencies/busy/npnc_sender.txt").unwrap();
        let mut magnetic_sender_handler =           FileHandler::new("inter_latencies/busy/magnetic_sender.txt").unwrap();
        let mut tokio_sender_handler =              FileHandler::new("inter_latencies/busy/tokio_sender.txt").unwrap();


        let (mut rtrb_tx, mut rtrb_rx) = rtrb::RingBuffer::<Data>::new(128);
        let (mut ringbuf_tx, mut ringbuf_rx) = ringbuf::HeapRb::<Data>::new(128).split();
        let (crossbeam_channel_tx, crossbeam_channel_rx) = crossbeam_channel::bounded::<Data>(128);
        let crossbeam_queue_tx = Arc::new(crossbeam_queue::ArrayQueue::<Data>::new(128));
        let crossbeam_queue_rx = Arc::clone(&crossbeam_queue_tx);
        let (kanal_tx, kanal_rx) = kanal::bounded::<Data>(128);
        let (flume_tx, flume_rx) = flume::bounded::<Data>(128);
        let (omango_tx, omango_rx) = omango::queue::mpmc::bounded::<Data>(128);
        let (npnc_tx, npnc_rx) = npnc::bounded::mpmc::channel::<Data>(128);
        let (magnetic_tx, magnetic_rx) = magnetic::mpsc::mpsc_queue(magnetic::buffer::dynamic::DynamicBuffer::<Data>::new(128).expect("Failed to create dynamic buffer"));
        let (tokio_tx, mut tokio_rx) = tokio::sync::mpsc::channel::<Data>(128);

        let mut rng = thread_rng();
        let mut random_data = vec![0; payload_size_in_bytes as usize];
        rng.fill_bytes(&mut random_data);


        let sender_experiment = experiment.clone();
        let sender_handle = std::thread::spawn(move || { // create a system thread
            let res = core_affinity::set_for_current(core_affinity::CoreId { id: cpu_0 }); // pin thread to specific core
            if res {
    
                // create tokio runtime
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .worker_threads(1)
                    .build()
                    .expect(&CL::Red.str("[!][Inter-Process][Busy] Failed to create runtime"));
                let local = tokio::task::LocalSet::new();
                local.block_on(&runtime, async {

                    // these experiments will be sending data to the receiver core every 1ms (this is the minimum of a tokio sleep command)

                    match sender_experiment {
                        InterProcessSync::Rtrb => {
                            for _ in 0..sample_size {

                                let send_time = minstant::Instant::now();
                                rtrb_tx.push(Data::new(random_data.clone(), false)).expect(&CL::Red.str("[!][Inter-Process][Busy] Rtrb | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                rtrb_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] Rtrb | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            rtrb_tx.push(Data::new(random_data.clone(), true)).expect(&CL::Red.str("[!][Inter-Process][Busy] Rtrb | Failed to send message"));
                        },
                        InterProcessSync::Ringbuf => {
                            for _ in 0..sample_size {

                                let send_time = minstant::Instant::now();
                                ringbuf_tx.push(Data::new(random_data.clone(), false)).expect(&CL::Red.str("[!][Inter-Process][Busy] Ringbuf | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                ringbuf_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] Ringbuf | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            ringbuf_tx.push(Data::new(random_data.clone(), true)).expect(&CL::Red.str("[!][Inter-Process][Busy] Ringbuf | Failed to send message"));
                        },
                        InterProcessSync::CrossbeamChannel => {
                            for _ in 0..sample_size {

                                let send_time = minstant::Instant::now();
                                crossbeam_channel_tx.send(Data::new(random_data.clone(), false)).expect(&CL::Red.str("[!][Inter-Process][Busy] CrossbeamChannel | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                crossbeam_channel_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] CrossbeamChannel | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            crossbeam_channel_tx.send(Data::new(random_data.clone(), true)).expect(&CL::Red.str("[!][Inter-Process][Busy] CrossbeamChannel | Failed to send message"));
                        },
                        InterProcessSync::CrossbeamQueue => {
                            for _ in 0..sample_size {

                                let send_time = minstant::Instant::now();
                                crossbeam_queue_tx.push(Data::new(random_data.clone(), false)).expect(&CL::Red.str("[!][Inter-Process][Busy] CrossbeamQueue | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                crossbeam_queue_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] CrossbeamQueue | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            crossbeam_queue_tx.push(Data::new(random_data.clone(), true)).expect(&CL::Red.str("[!][Inter-Process][Busy] CrossbeamQueue | Failed to send message"));
                        },
                        InterProcessSync::Kanal => {
                            for _ in 0..sample_size {

                                let send_time = minstant::Instant::now();
                                kanal_tx.send(Data::new(random_data.clone(), false)).expect(&CL::Red.str("[!][Inter-Process][Busy] Kanal | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                kanal_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] Kanal | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            kanal_tx.send(Data::new(random_data.clone(), true)).expect(&CL::Red.str("[!][Inter-Process][Busy] Kanal | Failed to send message"));
                        },
                        InterProcessSync::Flume => {
                            for _ in 0..sample_size {

                                let send_time = minstant::Instant::now();
                                flume_tx.send(Data::new(random_data.clone(), false)).expect(&CL::Red.str("[!][Inter-Process][Busy] Flume | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                flume_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] Flume | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            flume_tx.send(Data::new(random_data.clone(), true)).expect(&CL::Red.str("[!][Inter-Process][Busy] Flume | Failed to send message"));
                        },
                        InterProcessSync::Omango => {
                            for _ in 0..sample_size {

                                let send_time = minstant::Instant::now();
                                omango_tx.send(Data::new(random_data.clone(), false)).expect(&CL::Red.str("[!][Inter-Process][Busy] Omango | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                omango_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] Omango | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            omango_tx.send(Data::new(random_data.clone(), true)).expect(&CL::Red.str("[!][Inter-Process][Busy] Omango | Failed to send message"));
                        },
                        InterProcessSync::Npnc => {
                            for _ in 0..sample_size {

                                let send_time = minstant::Instant::now();
                                npnc_tx.produce(Data::new(random_data.clone(), false)).expect(&CL::Red.str("[!][Inter-Process][Busy] Npnc | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                npnc_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] Npnc | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            npnc_tx.produce(Data::new(random_data.clone(), true)).expect(&CL::Red.str("[!][Inter-Process][Busy] Npnc | Failed to send message"));
                        },
                        InterProcessSync::Magnetic => {
                            for _ in 0..sample_size {

                                let send_time = minstant::Instant::now();
                                magnetic_tx.push(Data::new(random_data.clone(), false)).expect(&CL::Red.str("[!][Inter-Process][Busy] Magnetic | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                magnetic_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] Magnetic | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            magnetic_tx.push(Data::new(random_data.clone(), true)).expect(&CL::Red.str("[!][Inter-Process][Busy] Magnetic | Failed to send message"));
                        },
                        InterProcessSync::Tokio => {
                            for _ in 0..sample_size {

                                let send_time = minstant::Instant::now();
                                tokio_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Inter-Process][Busy] Tokio | Failed to send message"));
                                let latency = send_time.elapsed().as_nanos();
                                tokio_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] Tokio | Failed to write to file"));

                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                            tokio_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Inter-Process][Busy] Tokio | Failed to send message"));
                        },
                    }
                });
            }
        });
    
        let mut rtrb_receiver_handler =               FileHandler::new("inter_latencies/busy/rtrb_receiver.txt").unwrap();
        let mut ringbuf_receiver_handler =            FileHandler::new("inter_latencies/busy/ringbuf_receiver.txt").unwrap();
        let mut crossbeam_channel_receiver_handler =  FileHandler::new("inter_latencies/busy/crossbeam_channel_receiver.txt").unwrap();
        let mut crossbeam_queue_receiver_handler =    FileHandler::new("inter_latencies/busy/crossbeam_queue_receiver.txt").unwrap();
        let mut kanal_receiver_handler =              FileHandler::new("inter_latencies/busy/kanal_receiver.txt").unwrap();
        let mut flume_receiver_handler =              FileHandler::new("inter_latencies/busy/flume_receiver.txt").unwrap();
        let mut omango_receiver_handler =             FileHandler::new("inter_latencies/busy/omango_receiver.txt").unwrap();
        let mut npnc_receiver_handler =               FileHandler::new("inter_latencies/busy/npnc_receiver.txt").unwrap();
        let mut magnetic_receiver_handler =           FileHandler::new("inter_latencies/busy/magnetic_receiver.txt").unwrap();
        let mut tokio_receiver_handler =              FileHandler::new("inter_latencies/busy/tokio_receiver.txt").unwrap();

        let receiver_handle = std::thread::spawn(move || { // create a system thread
            let res = core_affinity::set_for_current(core_affinity::CoreId { id: cpu_1 }); // pin thread to specific core
            if res {
    
                // create tokio runtime
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .worker_threads(1)
                    .build()
                    .expect(&CL::Red.str("[!][Inter-Process][Busy] Failed to create runtime"));
                let local = tokio::task::LocalSet::new();
                local.block_on(&runtime, async {

                    // these experiments will be busy spinning and consuming all the resources on the designated core

                    match experiment {
                        InterProcessSync::Rtrb => {
                            loop {
                                match rtrb_rx.pop() {
                                    Ok(data) => {
                                        if data.kill { break }

                                        let latency = data.time.elapsed().as_nanos();
                                        rtrb_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] Rtrb | Failed to write to file"));

                                    },
                                    Err(_) => {}, // error will populate if buffer is empty, which is expected as the buffer is being read faster than it's being written to
                                }
                            }
                        },
                        InterProcessSync::Ringbuf => {
                            loop {
                                match ringbuf_rx.pop() {
                                    Some(data) => {
                                        if data.kill { break }

                                        let latency = data.time.elapsed().as_nanos();
                                        ringbuf_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] Ringbuf | Failed to write to file"));

                                    },
                                    None => {}, 
                                }
                            }
                        },
                        InterProcessSync::CrossbeamChannel => {
                            loop {
                                match crossbeam_channel_rx.try_recv() {
                                    Ok(data) => {
                                        if data.kill { break }

                                        let latency = data.time.elapsed().as_nanos();
                                        crossbeam_channel_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] CrossbeamChannel | Failed to write to file"));

                                    },
                                    Err(_) => {},
                                }
                            }
                        },
                        InterProcessSync::CrossbeamQueue => {
                            loop {
                                match crossbeam_queue_rx.pop() {
                                    Some(data) => {
                                        if data.kill { break }

                                        let latency = data.time.elapsed().as_nanos();
                                        crossbeam_queue_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] CrossbeamQueue | Failed to write to file"));

                                    },
                                    None => {},
                                }
                            }
                        },
                        InterProcessSync::Kanal => {
                            loop {
                                match kanal_rx.try_recv() {
                                    Ok(optional_data) => {
                                        match optional_data {
                                            Some(data) => {
                                                if data.kill { break }

                                                let latency = data.time.elapsed().as_nanos();
                                                kanal_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] Kanal | Failed to write to file"));
                                            },
                                            None => {}
                                        }
                                    },
                                    Err(e) => {
                                        CL::Red.print(&format!("[!][Inter-Process][Busy] Kanal | Failed to receive message: {}", e));
                                        break;
                                    },
                                }
                            }
                        },
                        InterProcessSync::Flume => {
                            loop {
                                match flume_rx.recv() {
                                    Ok(data) => {
                                        if data.kill { break }

                                        let latency = data.time.elapsed().as_nanos();
                                        flume_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] Flume | Failed to write to file"));

                                    },
                                    Err(_) => {},
                                }
                            }
                        },
                        InterProcessSync::Omango => {
                            loop {
                                match omango_rx.recv() {
                                    Ok(data) => {
                                        if data.kill { break }

                                        let latency = data.time.elapsed().as_nanos();
                                        omango_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] Omango | Failed to write to file"));

                                    },
                                    Err(_) => {},
                                }
                            }
                        },
                        InterProcessSync::Npnc => {
                            loop {
                                match npnc_rx.consume() {
                                    Ok(data) => {
                                        if data.kill { break }

                                        let latency = data.time.elapsed().as_nanos();
                                        npnc_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] Npnc | Failed to write to file"));

                                    },
                                    Err(_) => {},
                                }
                            }
                        },
                        InterProcessSync::Magnetic => {
                            loop {
                                match magnetic_rx.pop() {
                                    Ok(data) => {
                                        if data.kill { break }

                                        let latency = data.time.elapsed().as_nanos();
                                        magnetic_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] Magnetic | Failed to write to file"));

                                    },
                                    Err(_) => {},
                                }
                            }
                        },
                        InterProcessSync::Tokio => {
                            loop {
                                match tokio_rx.try_recv() {
                                    Ok(data) => {
                                        if data.kill { break }

                                        let latency = data.time.elapsed().as_nanos();
                                        tokio_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Inter-Process][Busy] Tokio | Failed to write to file"));

                                    },
                                    Err(_) => {},
                                }
                            }
                        },
                    }
                });
            }
        });
    
        receiver_handle.join().unwrap();
        sender_handle.join().unwrap();
    }

    

    CL::DullGreen.print("[+][Inter-Process][Busy] Complete\n");
}


// =-= Intra-Process =-=-= //
// - Communication within the same tokio runtime
// - Testing different tokio structures (single-threaded (pinned to one core / thread) vs multi-threaded across all cores)

fn intra_process_async_multi_thread(sample_size: i64, payload_size_in_bytes: i64) {
    CL::Teal.print("[+][Intra-Process][Async-Multi-Thread] Starting");

    // this experiment differs a bit. instead of creating and pinning a thread to a core, we assume that people going the intra-process 
    // route will be using a runtime over multiple cores. so, we'll create a multi-threaded runtime which will create worker threads equal
    // to the number of cores on the machine. this way, we can simulate the intra-process async scenario.

    // create tokio runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Failed to create runtime"));
    //let local = tokio::task::LocalSet::new();
    runtime.block_on(async {

        let mut kanal_sender_handler =               FileHandler::new("intra_latencies/async_multi_thread/kanal_sender.txt").unwrap();
        let mut flume_sender_handler =               FileHandler::new("intra_latencies/async_multi_thread/flume_sender.txt").unwrap();
        let mut tokio_sender_handler =               FileHandler::new("intra_latencies/async_multi_thread/tokio_sender.txt").unwrap();
        let mut async_channel_sender_handler =       FileHandler::new("intra_latencies/async_multi_thread/async_channel_sender.txt").unwrap();
        let mut crossfire_sender_handler =           FileHandler::new("intra_latencies/async_multi_thread/crossfire_sender.txt").unwrap();
        let mut thingbuf_sender_handler =            FileHandler::new("intra_latencies/async_multi_thread/thingbuf_sender.txt").unwrap();
        let mut tachyonix_sender_handler =           FileHandler::new("intra_latencies/async_multi_thread/tachyonix_sender.txt").unwrap();
        let mut postage_sender_handler =             FileHandler::new("intra_latencies/async_multi_thread/postage_sender.txt").unwrap();
        let mut async_broadcast_sender_handler =     FileHandler::new("intra_latencies/async_multi_thread/async_broadcast_sender.txt").unwrap();

        let mut kanal_receiver_handler =             FileHandler::new("intra_latencies/async_multi_thread/kanal_receiver.txt").unwrap();
        let mut flume_receiver_handler =             FileHandler::new("intra_latencies/async_multi_thread/flume_receiver.txt").unwrap();
        let mut tokio_receiver_handler =             FileHandler::new("intra_latencies/async_multi_thread/tokio_receiver.txt").unwrap();
        let mut async_channel_receiver_handler =     FileHandler::new("intra_latencies/async_multi_thread/async_channel_receiver.txt").unwrap();
        let mut crossfire_receiver_handler =         FileHandler::new("intra_latencies/async_multi_thread/crossfire_receiver.txt").unwrap();
        let mut thingbuf_receiver_handler =          FileHandler::new("intra_latencies/async_multi_thread/thingbuf_receiver.txt").unwrap();
        let mut tachyonix_receiver_handler =         FileHandler::new("intra_latencies/async_multi_thread/tachyonix_receiver.txt").unwrap();
        let mut postage_receiver_handler =           FileHandler::new("intra_latencies/async_multi_thread/postage_receiver.txt").unwrap();
        let mut async_broadcast_receiver_handler =   FileHandler::new("intra_latencies/async_multi_thread/async_broadcast_receiver.txt").unwrap();

        let (kanal_tx, kanal_rx) = kanal::bounded_async::<Data>(128);
        let (flume_tx, flume_rx) = flume::bounded::<Data>(128);
        let (tokio_tx, mut tokio_rx) = tokio::sync::mpsc::channel::<Data>(128);
        let (async_channel_tx, async_channel_rx) = async_channel::bounded::<Data>(128);
        let (crossfire_tx, crossfire_rx) = crossfire::mpsc::bounded_future_both::<Data>(128);
        let (thingbuf_tx, thingbuf_rx) = thingbuf::mpsc::channel::<Data>(128);
        let (tachyonix_tx, mut tachyonix_rx) = tachyonix::channel::<Data>(128);
        let (mut postage_tx, mut postage_rx) = postage::mpsc::channel::<Data>(128);
        let (async_broadcast_tx, mut async_broadcast_rx) = async_broadcast::broadcast::<Data>(128);


        let mut rng = thread_rng();
        let mut random_data = vec![0; payload_size_in_bytes as usize];
        rng.fill_bytes(&mut random_data);


        let mut handles = Vec::new();

        let experiments = vec![
            AsyncExperiments::Kanal,
            AsyncExperiments::Flume,
            AsyncExperiments::Tokio,
            AsyncExperiments::AsyncChannel,
            AsyncExperiments::Crossfire,
            AsyncExperiments::Thingbuf,
            AsyncExperiments::Tachyonix,
            AsyncExperiments::Postage,
            AsyncExperiments::AsyncBroadcast,
        ];

        
        let sender_handler = tokio::task::spawn(async move {
            for experiment in experiments {
                CL::Dull.print_literal(format!("[-] {:?} | Running experiment...", experiment));
                match experiment {
                    AsyncExperiments::Kanal => {
                        for _ in 0..sample_size {

                            let send_time = minstant::Instant::now();
                            kanal_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Kanal | Failed to send message"));
                            let latency = send_time.elapsed().as_nanos();
                            kanal_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Kanal | Failed to write to file"));

                            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        }
                        kanal_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Kanal | Failed to send message"));
                    },
                    AsyncExperiments::Flume => {
                        for _ in 0..sample_size {

                            let send_time = minstant::Instant::now();
                            flume_tx.send_async(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Flume | Failed to send message"));
                            let latency = send_time.elapsed().as_nanos();
                            flume_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Flume | Failed to write to file"));

                            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        }
                        flume_tx.send_async(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Flume | Failed to send message"));
                    },
                    AsyncExperiments::Tokio => {
                        for _ in 0..sample_size {
                            let send_time = minstant::Instant::now();
                            tokio_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Tokio | Failed to send message"));
                            let latency = send_time.elapsed().as_nanos();
                            tokio_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Tokio | Failed to write to file"));

                            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        }
                        tokio_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Tokio | Failed to send message"));
                    },
                    AsyncExperiments::AsyncChannel => {
                        for _ in 0..sample_size {

                            let send_time = minstant::Instant::now();
                            async_channel_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Async Channel | Failed to send message"));
                            let latency = send_time.elapsed().as_nanos();
                            async_channel_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Async Channel | Failed to write to file"));

                            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        }
                        async_channel_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Async Channel | Failed to send message"));
                    },
                    AsyncExperiments::Crossfire => {
                        for _ in 0..sample_size {

                            let send_time = minstant::Instant::now();
                            crossfire_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Crossfire | Failed to send message"));
                            let latency = send_time.elapsed().as_nanos();
                            crossfire_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Crossfire | Failed to write to file"));

                            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        }
                        crossfire_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Crossfire | Failed to send message"));
                    },
                    AsyncExperiments::Thingbuf => {
                        for _ in 0..sample_size {

                            let send_time = minstant::Instant::now();
                            thingbuf_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Thingbuf | Failed to send message"));
                            let latency = send_time.elapsed().as_nanos();
                            thingbuf_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Thingbuf | Failed to write to file"));

                            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        }
                        thingbuf_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Thingbuf | Failed to send message"));
                    },
                    AsyncExperiments::Tachyonix => {
                        for _ in 0..sample_size {

                            let send_time = minstant::Instant::now();
                            tachyonix_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Tachyonix | Failed to send message"));
                            let latency = send_time.elapsed().as_nanos();
                            tachyonix_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Tachyonix | Failed to write to file"));

                            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        }
                        tachyonix_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Tachyonix | Failed to send message"));
                    },
                    AsyncExperiments::Postage => {
                        for _ in 0..sample_size {

                            let send_time = minstant::Instant::now();
                            postage_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Postage | Failed to send message"));
                            let latency = send_time.elapsed().as_nanos();
                            postage_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Postage | Failed to write to file"));

                            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        }
                        postage_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Postage | Failed to send message"));
                    },
                    AsyncExperiments::AsyncBroadcast => {
                        for _ in 0..sample_size {

                            let send_time = minstant::Instant::now();
                            async_broadcast_tx.broadcast(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Async Broadcast | Failed to send message"));
                            let latency = send_time.elapsed().as_nanos();
                            async_broadcast_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Async Broadcast | Failed to write to file"));

                            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        }
                        async_broadcast_tx.broadcast(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Async Broadcast | Failed to send message"));
                    },
                }
            }
        });
        handles.push(sender_handler);



        

        let kanal_handler = tokio::task::spawn(async move {
            loop {
                match kanal_rx.recv().await {
                    Ok(data) => {
                        if data.kill { break }

                        let latency = data.time.elapsed().as_nanos();
                        kanal_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Kanal | Failed to write to file"));

                    },
                    Err(e) => {
                        CL::Red.print(&format!("[!][Intra-Process][Async-Multi-Thread] Kanal | Failed to receive message: {}", e));
                        break;
                    }
                }
            }
        });
        handles.push(kanal_handler);

        let flume_handler = tokio::task::spawn(async move {
            loop {
                match flume_rx.recv_async().await {
                    Ok(data) => {
                        if data.kill { break }

                        let latency = data.time.elapsed().as_nanos();
                        flume_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Flume | Failed to write to file"));

                    },
                    Err(e) => {
                        CL::Red.print(&format!("[!][Intra-Process][Async-Multi-Thread] Flume | Failed to receive message: {}", e));
                        break;
                    }
                }
            }
        });
        handles.push(flume_handler);

        let tokio_handler = tokio::task::spawn(async move {
            while let Some(data) = tokio_rx.recv().await {
                if data.kill { break }

                let latency = data.time.elapsed().as_nanos();
                tokio_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Tokio | Failed to write to file"));
            }
        });
        handles.push(tokio_handler);

        let async_channel_handler = tokio::task::spawn(async move {
            loop {
                match async_channel_rx.recv().await {
                    Ok(data) => {
                        if data.kill { break }

                        let latency = data.time.elapsed().as_nanos();
                        async_channel_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Async Channel | Failed to write to file"));

                    },
                    Err(e) => {
                        CL::Red.print(&format!("[!][Intra-Process][Async-Multi-Thread] Async Channel | Failed to receive message: {}", e));
                        break;
                    }
                }
            }
        });
        handles.push(async_channel_handler);

        let crossfire_handler = tokio::task::spawn(async move {
            loop {
                match crossfire_rx.recv().await {
                    Ok(data) => {
                        if data.kill { break }

                        let latency = data.time.elapsed().as_nanos();
                        crossfire_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Crossfire | Failed to write to file"));

                    },
                    Err(e) => {
                        CL::Red.print(&format!("[!][Intra-Process][Async-Multi-Thread] Crossfire | Failed to receive message: {}", e));
                        break;
                    }
                }
            }
        });
        handles.push(crossfire_handler);

        let thingbuf_handler = tokio::task::spawn(async move {
            loop {
                match thingbuf_rx.recv().await {
                    Some(data) => {
                        if data.kill { break }

                        let latency = data.time.elapsed().as_nanos();
                        thingbuf_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Thingbuf | Failed to write to file"));

                    },
                    None => {
                        CL::Red.print(&format!("[!][Intra-Process][Async-Multi-Thread] Thingbuf | Failed to receive message"));
                        break;
                    }
                }
            }
        });
        handles.push(thingbuf_handler);

        let tachyonix_handler = tokio::task::spawn(async move {
            loop {
                match tachyonix_rx.recv().await {
                    Ok(data) => {
                        if data.kill { break }

                        let latency = data.time.elapsed().as_nanos();
                        tachyonix_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Tachyonix | Failed to write to file"));

                    },
                    Err(e) => {
                        CL::Red.print(&format!("[!][Intra-Process][Async-Multi-Thread] Tachyonix | Failed to receive message: {}", e));
                        break;
                    }
                }
            }
        });
        handles.push(tachyonix_handler);

        let postage_handler = tokio::task::spawn(async move {
            loop {
                match postage_rx.recv().await {
                    Some(data) => {
                        if data.kill { break }

                        let latency = data.time.elapsed().as_nanos();
                        postage_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Postage | Failed to write to file"));

                    },
                    None => {
                        CL::Red.print(&format!("[!][Intra-Process][Async-Multi-Thread] Postage | Failed to receive message"));
                        break;
                    }
                }
            }
        });
        handles.push(postage_handler);

        let async_broadcast_handler = tokio::task::spawn(async move {
            loop {
                match async_broadcast_rx.recv().await {
                    Ok(data) => {
                        if data.kill { break }

                        let latency = data.time.elapsed().as_nanos();
                        async_broadcast_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Async Broadcast | Failed to write to file"));

                    },
                    Err(e) => {
                        CL::Red.print(&format!("[!][Intra-Process][Async-Multi-Thread] Async Broadcast | Failed to receive message: {}", e));
                        break;
                    }
                }
            }
        });
        handles.push(async_broadcast_handler);

        for handle in handles {
            handle.await.expect(&CL::Red.str("[!][Intra-Process][Async-Multi-Thread] Receiver_handle | Failed to await a handle"));
        }

    });



    CL::DullGreen.print("[+][Intra-Process][Async-Multi-Thread] Complete\n");

}

fn intra_process_async_single_thread(sample_size: i64, payload_size_in_bytes: i64, cpu_0: usize) {
    CL::Teal.print("[+][Intra-Process][Async-Single-Thread] Starting");

    let sender_handle = std::thread::spawn(move || { // create a system thread
        let res = core_affinity::set_for_current(core_affinity::CoreId { id: cpu_0 }); // pin thread to specific core
        if res {
            // create tokio runtime
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Failed to create runtime"));
            let local = tokio::task::LocalSet::new();
            local.block_on(&runtime, async {

                let mut kanal_sender_handler =               FileHandler::new("intra_latencies/async_single_thread/kanal_sender.txt").unwrap();
                let mut flume_sender_handler =               FileHandler::new("intra_latencies/async_single_thread/flume_sender.txt").unwrap();
                let mut tokio_sender_handler =               FileHandler::new("intra_latencies/async_single_thread/tokio_sender.txt").unwrap();
                let mut async_channel_sender_handler =       FileHandler::new("intra_latencies/async_single_thread/async_channel_sender.txt").unwrap();
                let mut crossfire_sender_handler =           FileHandler::new("intra_latencies/async_single_thread/crossfire_sender.txt").unwrap();
                let mut thingbuf_sender_handler =            FileHandler::new("intra_latencies/async_single_thread/thingbuf_sender.txt").unwrap();
                let mut tachyonix_sender_handler =           FileHandler::new("intra_latencies/async_single_thread/tachyonix_sender.txt").unwrap();
                let mut postage_sender_handler =             FileHandler::new("intra_latencies/async_single_thread/postage_sender.txt").unwrap();
                let mut async_broadcast_sender_handler =     FileHandler::new("intra_latencies/async_single_thread/async_broadcast_sender.txt").unwrap();

                let mut kanal_receiver_handler =             FileHandler::new("intra_latencies/async_single_thread/kanal_receiver.txt").unwrap();
                let mut flume_receiver_handler =             FileHandler::new("intra_latencies/async_single_thread/flume_receiver.txt").unwrap();
                let mut tokio_receiver_handler =             FileHandler::new("intra_latencies/async_single_thread/tokio_receiver.txt").unwrap();
                let mut async_channel_receiver_handler =     FileHandler::new("intra_latencies/async_single_thread/async_channel_receiver.txt").unwrap();
                let mut crossfire_receiver_handler =         FileHandler::new("intra_latencies/async_single_thread/crossfire_receiver.txt").unwrap();
                let mut thingbuf_receiver_handler =          FileHandler::new("intra_latencies/async_single_thread/thingbuf_receiver.txt").unwrap();
                let mut tachyonix_receiver_handler =         FileHandler::new("intra_latencies/async_single_thread/tachyonix_receiver.txt").unwrap();
                let mut postage_receiver_handler =           FileHandler::new("intra_latencies/async_single_thread/postage_receiver.txt").unwrap();
                let mut async_broadcast_receiver_handler =   FileHandler::new("intra_latencies/async_single_thread/async_broadcast_receiver.txt").unwrap();

                let (kanal_tx, kanal_rx) = kanal::bounded_async::<Data>(128);
                let (flume_tx, flume_rx) = flume::bounded::<Data>(128);
                let (tokio_tx, mut tokio_rx) = tokio::sync::mpsc::channel::<Data>(128);
                let (async_channel_tx, async_channel_rx) = async_channel::bounded::<Data>(128);
                let (crossfire_tx, crossfire_rx) = crossfire::mpsc::bounded_future_both::<Data>(128);
                let (thingbuf_tx, thingbuf_rx) = thingbuf::mpsc::channel::<Data>(128);
                let (tachyonix_tx, mut tachyonix_rx) = tachyonix::channel::<Data>(128);
                let (mut postage_tx, mut postage_rx) = postage::mpsc::channel::<Data>(128);
                let (async_broadcast_tx, mut async_broadcast_rx) = async_broadcast::broadcast::<Data>(128);


                let mut rng = thread_rng();
                let mut random_data = vec![0; payload_size_in_bytes as usize];
                rng.fill_bytes(&mut random_data);


                let mut handles = Vec::new();

                let experiments = vec![
                    AsyncExperiments::Kanal,
                    AsyncExperiments::Flume,
                    AsyncExperiments::Tokio,
                    AsyncExperiments::AsyncChannel,
                    AsyncExperiments::Crossfire,
                    AsyncExperiments::Thingbuf,
                    AsyncExperiments::Tachyonix,
                    AsyncExperiments::Postage,
                    AsyncExperiments::AsyncBroadcast,
                ];

                
                let sender_handler = tokio::task::spawn_local(async move {
                    for experiment in experiments {
                        CL::Dull.print_literal(format!("[-] {:?} | Running experiment...", experiment));
                        match experiment {
                            AsyncExperiments::Kanal => {
                                for _ in 0..sample_size {

                                    let send_time = minstant::Instant::now();
                                    kanal_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Kanal | Failed to send message"));
                                    let latency = send_time.elapsed().as_nanos();
                                    kanal_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Kanal | Failed to write to file"));

                                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                                }
                                kanal_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Kanal | Failed to send message"));
                            },
                            AsyncExperiments::Flume => {
                                for _ in 0..sample_size {

                                    let send_time = minstant::Instant::now();
                                    flume_tx.send_async(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Flume | Failed to send message"));
                                    let latency = send_time.elapsed().as_nanos();
                                    flume_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Flume | Failed to write to file"));

                                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                                }
                                flume_tx.send_async(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Flume | Failed to send message"));
                            },
                            AsyncExperiments::Tokio => {
                                for _ in 0..sample_size {
                                    let send_time = minstant::Instant::now();
                                    tokio_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Tokio | Failed to send message"));
                                    let latency = send_time.elapsed().as_nanos();
                                    tokio_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Tokio | Failed to write to file"));

                                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                                }
                                tokio_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Tokio | Failed to send message"));
                            },
                            AsyncExperiments::AsyncChannel => {
                                for _ in 0..sample_size {

                                    let send_time = minstant::Instant::now();
                                    async_channel_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Async Channel | Failed to send message"));
                                    let latency = send_time.elapsed().as_nanos();
                                    async_channel_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Async Channel | Failed to write to file"));

                                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                                }
                                async_channel_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Async Channel | Failed to send message"));
                            },
                            AsyncExperiments::Crossfire => {
                                for _ in 0..sample_size {

                                    let send_time = minstant::Instant::now();
                                    crossfire_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Crossfire | Failed to send message"));
                                    let latency = send_time.elapsed().as_nanos();
                                    crossfire_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Crossfire | Failed to write to file"));

                                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                                }
                                crossfire_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Crossfire | Failed to send message"));
                            },
                            AsyncExperiments::Thingbuf => {
                                for _ in 0..sample_size {

                                    let send_time = minstant::Instant::now();
                                    thingbuf_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Thingbuf | Failed to send message"));
                                    let latency = send_time.elapsed().as_nanos();
                                    thingbuf_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Thingbuf | Failed to write to file"));

                                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                                }
                                thingbuf_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Thingbuf | Failed to send message"));
                            },
                            AsyncExperiments::Tachyonix => {
                                for _ in 0..sample_size {

                                    let send_time = minstant::Instant::now();
                                    tachyonix_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Tachyonix | Failed to send message"));
                                    let latency = send_time.elapsed().as_nanos();
                                    tachyonix_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Tachyonix | Failed to write to file"));

                                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                                }
                                tachyonix_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Tachyonix | Failed to send message"));
                            },
                            AsyncExperiments::Postage => {
                                for _ in 0..sample_size {

                                    let send_time = minstant::Instant::now();
                                    postage_tx.send(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Postage | Failed to send message"));
                                    let latency = send_time.elapsed().as_nanos();
                                    postage_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Postage | Failed to write to file"));

                                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                                }
                                postage_tx.send(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Postage | Failed to send message"));
                            },
                            AsyncExperiments::AsyncBroadcast => {
                                for _ in 0..sample_size {

                                    let send_time = minstant::Instant::now();
                                    async_broadcast_tx.broadcast(Data::new(random_data.clone(), false)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Async Broadcast | Failed to send message"));
                                    let latency = send_time.elapsed().as_nanos();
                                    async_broadcast_sender_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Async Broadcast | Failed to write to file"));

                                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                                }
                                async_broadcast_tx.broadcast(Data::new(random_data.clone(), true)).await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Async Broadcast | Failed to send message"));
                            },
                        }
                    }
                });
                handles.push(sender_handler);



                

                let kanal_handler = tokio::task::spawn_local(async move {
                    loop {
                        match kanal_rx.recv().await {
                            Ok(data) => {
                                if data.kill { break }

                                let latency = data.time.elapsed().as_nanos();
                                kanal_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Kanal | Failed to write to file"));

                            },
                            Err(e) => {
                                CL::Red.print(&format!("[!][Intra-Process][Async-Single-Thread] Kanal | Failed to receive message: {}", e));
                                break;
                            }
                        }
                    }
                });
                handles.push(kanal_handler);

                let flume_handler = tokio::task::spawn_local(async move {
                    loop {
                        match flume_rx.recv_async().await {
                            Ok(data) => {
                                if data.kill { break }

                                let latency = data.time.elapsed().as_nanos();
                                flume_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Flume | Failed to write to file"));

                            },
                            Err(e) => {
                                CL::Red.print(&format!("[!][Intra-Process][Async-Single-Thread] Flume | Failed to receive message: {}", e));
                                break;
                            }
                        }
                    }
                });
                handles.push(flume_handler);

                let tokio_handler = tokio::task::spawn_local(async move {
                    while let Some(data) = tokio_rx.recv().await {
                        if data.kill { break }

                        let latency = data.time.elapsed().as_nanos();
                        tokio_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Tokio | Failed to write to file"));
                    }
                });
                handles.push(tokio_handler);

                let async_channel_handler = tokio::task::spawn_local(async move {
                    loop {
                        match async_channel_rx.recv().await {
                            Ok(data) => {
                                if data.kill { break }

                                let latency = data.time.elapsed().as_nanos();
                                async_channel_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Async Channel | Failed to write to file"));

                            },
                            Err(e) => {
                                CL::Red.print(&format!("[!][Intra-Process][Async-Single-Thread] Async Channel | Failed to receive message: {}", e));
                                break;
                            }
                        }
                    }
                });
                handles.push(async_channel_handler);

                let crossfire_handler = tokio::task::spawn_local(async move {
                    loop {
                        match crossfire_rx.recv().await {
                            Ok(data) => {
                                if data.kill { break }

                                let latency = data.time.elapsed().as_nanos();
                                crossfire_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Crossfire | Failed to write to file"));

                            },
                            Err(e) => {
                                CL::Red.print(&format!("[!][Intra-Process][Async-Single-Thread] Crossfire | Failed to receive message: {}", e));
                                break;
                            }
                        }
                    }
                });
                handles.push(crossfire_handler);

                let thingbuf_handler = tokio::task::spawn_local(async move {
                    loop {
                        match thingbuf_rx.recv().await {
                            Some(data) => {
                                if data.kill { break }

                                let latency = data.time.elapsed().as_nanos();
                                thingbuf_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Thingbuf | Failed to write to file"));

                            },
                            None => {
                                CL::Red.print(&format!("[!][Intra-Process][Async-Single-Thread] Thingbuf | Failed to receive message"));
                                break;
                            }
                        }
                    }
                });
                handles.push(thingbuf_handler);

                let tachyonix_handler = tokio::task::spawn_local(async move {
                    loop {
                        match tachyonix_rx.recv().await {
                            Ok(data) => {
                                if data.kill { break }

                                let latency = data.time.elapsed().as_nanos();
                                tachyonix_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Tachyonix | Failed to write to file"));

                            },
                            Err(e) => {
                                CL::Red.print(&format!("[!][Intra-Process][Async-Single-Thread] Tachyonix | Failed to receive message: {}", e));
                                break;
                            }
                        }
                    }
                });
                handles.push(tachyonix_handler);

                let postage_handler = tokio::task::spawn_local(async move {
                    loop {
                        match postage_rx.recv().await {
                            Some(data) => {
                                if data.kill { break }

                                let latency = data.time.elapsed().as_nanos();
                                postage_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Postage | Failed to write to file"));

                            },
                            None => {
                                CL::Red.print(&format!("[!][Intra-Process][Async-Single-Thread] Postage | Failed to receive message"));
                                break;
                            }
                        }
                    }
                });
                handles.push(postage_handler);

                let async_broadcast_handler = tokio::task::spawn_local(async move {
                    loop {
                        match async_broadcast_rx.recv().await {
                            Ok(data) => {
                                if data.kill { break }

                                let latency = data.time.elapsed().as_nanos();
                                async_broadcast_receiver_handler.write_line(latency.to_string()).expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Async Broadcast | Failed to write to file"));

                            },
                            Err(e) => {
                                CL::Red.print(&format!("[!][Intra-Process][Async-Single-Thread] Async Broadcast | Failed to receive message: {}", e));
                                break;
                            }
                        }
                    }
                });
                handles.push(async_broadcast_handler);

                for handle in handles {
                    handle.await.expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Receiver_handle | Failed to await a handle"));
                }

            });
        }
    });

    sender_handle.join().expect(&CL::Red.str("[!][Intra-Process][Async-Single-Thread] Sender_handle | Failed to join a handle"));

    CL::DullGreen.print("[+][Intra-Process][Async-Single-Thread] Complete\n");

}



fn main() {
    CL::DullTeal.print("\n
    //////////////////////////////////////
    //                                  //
    //    Research Project Overview     //
    //                                  //
    // ================================ //
    // Objectives:                      //
    // -------------------------------- //
    // Inter-process communication      //
    // - async                          //
    // - busy spinning (synchronous)    //
    // -------------------------------- //
    // Intra-process communication      //
    // - async (single-thread)          //
    // - async (multi-thread)           //
    // ================================ //
    //////////////////////////////////////\n\n
    ");

    // =------------------------------------------------------------------------------= //

    let sample_size: i64 = 10000; // Number of samples to take for each test
    let payload_size_in_bytes: i64 = 0; // 0 bytes is default

    let cpu_0: usize = 0; // this one is always the sender in the experiments
    let cpu_1: usize = 1; // this one is always the receiver in the experiments

    // =------------------------------------------------------------------------------= //

    CL::Green.print("[+][Main] Running all tests\n");

    inter_process_async(sample_size, payload_size_in_bytes, cpu_0, cpu_1);
    inter_process_busy_spin(sample_size, payload_size_in_bytes, cpu_0, cpu_1);
    intra_process_async_multi_thread(sample_size, payload_size_in_bytes);
    intra_process_async_single_thread(sample_size, payload_size_in_bytes, cpu_0); // cpu_0 is the whole CPU being tested on

    CL::Green.print("[+][Main] All tests complete\n");


}