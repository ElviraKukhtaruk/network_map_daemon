10 1200
20 1300 +100
30 1400 +200
----- reboot
40 1900 +700 +1900

10 500
20 300
30 450
40 4000

3498675983475932465834675697834659
0

// Tohle jo
20:00 500 mbit
20:01 400 mbit
20:02 350 mbit
20:03 490 mbit

//Tohle ne
20:00 500 mbit
20:01 900 mbit
20:02 1250 mbit
20:03 1740 mbit

int cur_rx_val = 0;
int last_rx_val = 0;

loop {
    curr_rx_val = get_rx_stat(interface); -> 1000 | 3000 | (After death) 6000
    write_to_clickhouse(interface, curr_rx_val - last_rx_val); -> 1000 - 0 = 1000 | 3000 - 1000 = 2000 | 6000 - 0 = 6000
    last_rx_val = cur_rx_val; -> 1000 | 2000 (DIED 💀) | 6000
    
    sleep(1000);
}

=====================

int last_rx_val = get_rx_stat(interface); -> 1000 | 3000 | (After death) 6000

loop {
    sleep(1000);

    int curr_rx_val = get_rx_stat(interface); -> 3000 | (DIED 💀) | 7000
    write_to_clickhouse(interface, curr_rx_val - last_rx_val); -> 3000 - 1000 = 2000 | 7000 - 6000 = 1000
    last_rx_val = cur_rx_val; -> 3000 | 1000
}

------------------------

Interfaces configurovat v configu pomoci Regex.


    
