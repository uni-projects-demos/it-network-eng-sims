# IT Network Engineering Simulations

IT network engineering simulations ported to Rust/WebAssembly for interactive web demos with SvelteKit UI:

* <details><summary>TCP/UDP File Transfer</summary>

    ## TCP/UDP

    Python Transmission Control Protocol (TCP) & User Datagram Protocol (UDP) applications for simulating sequential byte-file transfer between sender, receiver & network channel socket interfaces.

    ### TCP Example

    The expected packet loss rate is `N/((1-P)(1-P))`, where `N` & `P` are independent packet bit error & loss probabilities, respectively. Here, `"Hello World!"` is transmitted with `N = 0.1` & `P = 0.5`:

    #### Channel console output

    ```text
    Waiting for connection...
    Listening for socket connection to receiver...
    In-port for receiver connected.
    Out-port for receiver connected.
    Listening for socket connection to sender...
    In-port for sender connected.
    Out-port for sender connected.
    All sockets are connected

    Transmission status report: 
    Data packet containing 13 chars transmitted from sender
    A μ value of 0.3575 < 0.5, indicating #1 occurrence of the probability of a packet loss event.
    Data packet containing 13 chars transmitted from sender
    Data packet containing 13 chars transmitted to receiver
    Acknowledgement packet containing 0 chars transmitted from receiver
    Acknowledgement packet containing 0 chars transmitted to sender
    Data packet containing 0 chars transmitted from sender
    A μ value of 0.4546 < 0.5, indicating #2 occurrence of the probability of a packet loss event.
    Data packet containing 0 chars transmitted from sender
    A μ value of 0.329 < 0.5, indicating #3 occurrence of the probability of a packet loss event.
    Data packet containing 0 chars transmitted from sender
    A μ value of 0.3039 < 0.5, indicating #4 occurrence of the probability of a packet loss event.
    Data packet containing 0 chars transmitted from sender
    A μ value of 0.4606 < 0.5, indicating #5 occurrence of the probability of a packet loss event.
    Data packet containing 0 chars transmitted from sender
    A μ value of 0.2948 < 0.5, indicating #6 occurrence of the probability of a packet loss event.
    Data packet containing 0 chars transmitted from sender
    Data packet containing 0 chars transmitted to receiver
    Acknowledgement packet containing 0 chars transmitted from receiver
    Acknowledgement packet containing 0 chars transmitted to sender
    All TCP transmissions complete.

    Successful transmission of 4 packets in 6.02 seconds:
    - 10 transmissions sent
    - 10 transmissions received
    - 6 packets lost at a probability of 50.0%
    - 0 bit errors at a probability of 10.0%.

    Connection #1 successfully closed.
    Connection #2 successfully closed.
    Socket #1 successfully closed.
    Socket #2 successfully closed.
    Socket #3 successfully closed.
    Socket #4 successfully closed.
    TCP-channel: transmission complete.
    ```

    #### Receiver console output

    ```text
    Waiting for connection...
    Out-port for channel connected.
    Listening for socket connection to channel...
    In-port for channel connected.
    All sockets are connected

    Transmission status report: 

    Data packet #1 transmission success. 13-char data packet contents:

    Hello World!


    Data packet #2 transmission success. 0-char data packet contents:

    <empty packet>

    Successful transmission of 2 packets:
    - 2 transmissions sent
    - 2 transmissions received.

    received_send.txt successfully closed
    Connection #1 successfully closed.
    Socket #1 successfully closed.
    Socket #2 successfully closed.
    TCP-receiver: transmission complete.
    ```

    #### Sender console output

    ```text
    Waiting for connection...
    Out-port for channel connected.
    Listening for socket connection to channel...
    In-port for channel connected.
    All sockets are connected

    Transmission status report: 

    Data packet #1 transmission success after 2 attempt(s). 13-char data packet contents:

    Hello World!


    Data packet #2 transmission success after 6 attempt(s). 0-char data packet contents:

    <empty packet>

    Successful transmission of 2 packets:
    - 8 transmissions sent
    - 2 transmissions received.

    send.txt successfully closed
    Connection #1 successfully closed.
    Socket #1 successfully closed.
    Socket #2 successfully closed.
    TCP-sender: transmission complete.
    ```

    ### TCP (University of Canterbury) Project Contributors/Collaborators

    * [Adam Ross](https://github.com/r055a)
    * Helen Yang

    ### UDP (University of Canterbury) Project Collaborators/Contributors

    * [Jorge Leonard-Piñero](https://github.com/jorgitoxo)
    * [Adam Ross](https://github.com/r055a)
  </details>
* <details><summary>RIP Routing</summary>

    ## RIP

    Python application for simulating Routing Information Protocol (RIP).

    ### RIP Example

    RIP simulation on a 4-node network with router #3 inactived. Router #1 console output only.

    #### Router #1 console output

    ```bash
    RIP router program. Awaiting initialization...
    Router 1 has initialized successfully

    ----ROUTER 1 INITIALIZATION CONFIRMATION----
    Router 1 ports: [1774, 1040, 1041]
    Router 1 sockets and bounded ports: 
    (2, <socket.socket fd=5, family=2, type=2, proto=0, laddr=('127.0.0.1', 1774)>)
    (3, <socket.socket fd=6, family=2, type=2, proto=0, laddr=('127.0.0.1', 1040)>)
    (4, <socket.socket fd=7, family=2, type=2, proto=0, laddr=('127.0.0.1', 1041)>)

    ###########################################
    #|  ROUTER #1 NEIGHBOUR ROUTER(S) STATUS |#
    #|---------------------------------------|#
    #| ID | Port | Metric | Active | Garbage |#
    #|---------------------------------------|#
    #|  2 | 1025 |    1   |  True  |  False  |#
    #|  3 | 1035 |    3   |  True  |  False  |#
    #|  4 | 9035 |    1   |  True  |  False  |#
    ###########################################

    ######################################
    #|     ROUTER #1 ROUTING TABLE      |#
    #|----------------------------------|#
    #| Destination | First Hop | Metric |#
    #|----------------------------------|#
    #|      2      |    2      |    1   |#
    #|      3      |    4      |    2   |#
    #|      4      |    4      |    1   |#
    ######################################

    Router 2 is unresponsive for 4s

    ###########################################
    #|  ROUTER #1 NEIGHBOUR ROUTER(S) STATUS |#
    #|---------------------------------------|#
    #| ID | Port | Metric | Active | Garbage |#
    #|---------------------------------------|#
    #|  2 | 1025 |    1   | False  |  False  |#
    #|  3 | 1035 |    3   |  True  |  False  |#
    #|  4 | 9035 |    1   |  True  |  False  |#
    ###########################################

    ######################################
    #|     ROUTER #1 ROUTING TABLE      |#
    #|----------------------------------|#
    #| Destination | First Hop | Metric |#
    #|----------------------------------|#
    #|      2      |    2      |   16   |#
    #|      3      |    4      |    2   |#
    #|      4      |    4      |    1   |#
    ######################################

    Triggered routing table updates to all neighbours at 3.16s

    Router 3 is unresponsive for 4s

    ###########################################
    #|  ROUTER #1 NEIGHBOUR ROUTER(S) STATUS |#
    #|---------------------------------------|#
    #| ID | Port | Metric | Active | Garbage |#
    #|---------------------------------------|#
    #|  2 | 1025 |    1   | False  |  False  |#
    #|  3 | 1035 |    3   | False  |  False  |#
    #|  4 | 9035 |    1   |  True  |  False  |#
    ###########################################

    ######################################
    #|     ROUTER #1 ROUTING TABLE      |#
    #|----------------------------------|#
    #| Destination | First Hop | Metric |#
    #|----------------------------------|#
    #|      2      |    2      |   16   |#
    #|      3      |    4      |    2   |#
    #|      4      |    4      |    1   |#
    ######################################

    Triggered routing table updates to all neighbours at 3.16s

    Router 4 is unresponsive for 4s

    ###########################################
    #|  ROUTER #1 NEIGHBOUR ROUTER(S) STATUS |#
    #|---------------------------------------|#
    #| ID | Port | Metric | Active | Garbage |#
    #|---------------------------------------|#
    #|  2 | 1025 |    1   | False  |  False  |#
    #|  3 | 1035 |    3   | False  |  False  |#
    #|  4 | 9035 |    1   | False  |  False  |#
    ###########################################

    ######################################
    #|     ROUTER #1 ROUTING TABLE      |#
    #|----------------------------------|#
    #| Destination | First Hop | Metric |#
    #|----------------------------------|#
    #|      2      |    2      |   16   |#
    #|      3      |    4      |   16   |#
    #|      4      |    4      |   16   |#
    ######################################

    Triggered routing table updates to all neighbours at 3.16s

    ###########################################
    #|  ROUTER #1 NEIGHBOUR ROUTER(S) STATUS |#
    #|---------------------------------------|#
    #| ID | Port | Metric | Active | Garbage |#
    #|---------------------------------------|#
    #|  2 | 1025 |    1   |  True  |  False  |#
    #|  3 | 1035 |    3   | False  |  False  |#
    #|  4 | 9035 |    1   | False  |  False  |#
    ###########################################

    ###########################################
    #|  ROUTER #1 NEIGHBOUR ROUTER(S) STATUS |#
    #|---------------------------------------|#
    #| ID | Port | Metric | Active | Garbage |#
    #|---------------------------------------|#
    #|  2 | 1025 |    1   |  True  |  False  |#
    #|  3 | 1035 |    3   |  True  |  False  |#
    #|  4 | 9035 |    1   | False  |  False  |#
    ###########################################

    ###########################################
    #|  ROUTER #1 NEIGHBOUR ROUTER(S) STATUS |#
    #|---------------------------------------|#
    #| ID | Port | Metric | Active | Garbage |#
    #|---------------------------------------|#
    #|  2 | 1025 |    1   |  True  |  False  |#
    #|  3 | 1035 |    3   |  True  |  False  |#
    #|  4 | 9035 |    1   |  True  |  False  |#
    ###########################################

    ######################################
    #|     ROUTER #1 ROUTING TABLE      |#
    #|----------------------------------|#
    #| Destination | First Hop | Metric |#
    #|----------------------------------|#
    #|      2      |    2      |    1   |#
    #|      3      |    4      |    2   |#
    #|      4      |    4      |    1   |#
    ######################################

    Router 3 is unresponsive for 4s

    ###########################################
    #|  ROUTER #1 NEIGHBOUR ROUTER(S) STATUS |#
    #|---------------------------------------|#
    #| ID | Port | Metric | Active | Garbage |#
    #|---------------------------------------|#
    #|  2 | 1025 |    1   |  True  |  False  |#
    #|  3 | 1035 |    3   | False  |  False  |#
    #|  4 | 9035 |    1   |  True  |  False  |#
    ###########################################

    ######################################
    #|     ROUTER #1 ROUTING TABLE      |#
    #|----------------------------------|#
    #| Destination | First Hop | Metric |#
    #|----------------------------------|#
    #|      2      |    2      |    1   |#
    #|      3      |    4      |    2   |#
    #|      4      |    4      |    1   |#
    ######################################

    Triggered routing table updates to all neighbours at 0.96s

    ######################################
    #|     ROUTER #1 ROUTING TABLE      |#
    #|----------------------------------|#
    #| Destination | First Hop | Metric |#
    #|----------------------------------|#
    #|      2      |    2      |    1   |#
    #|      3      |    3      |   16   |#
    #|      4      |    4      |    1   |#
    ######################################

    Router 2 is unresponsive for 4s

    ###########################################
    #|  ROUTER #1 NEIGHBOUR ROUTER(S) STATUS |#
    #|---------------------------------------|#
    #| ID | Port | Metric | Active | Garbage |#
    #|---------------------------------------|#
    #|  2 | 1025 |    1   | False  |  False  |#
    #|  3 | 1035 |    3   | False  |  False  |#
    #|  4 | 9035 |    1   |  True  |  False  |#
    ###########################################

    ######################################
    #|     ROUTER #1 ROUTING TABLE      |#
    #|----------------------------------|#
    #| Destination | First Hop | Metric |#
    #|----------------------------------|#
    #|      2      |    2      |   16   |#
    #|      3      |    3      |   16   |#
    #|      4      |    4      |    1   |#
    ######################################

    Triggered routing table updates to all neighbours at 0.92s

    ###########################################
    #|  ROUTER #1 NEIGHBOUR ROUTER(S) STATUS |#
    #|---------------------------------------|#
    #| ID | Port | Metric | Active | Garbage |#
    #|---------------------------------------|#
    #|  2 | 1025 |    1   |  True  |  False  |#
    #|  3 | 1035 |    3   | False  |  False  |#
    #|  4 | 9035 |    1   |  True  |  False  |#
    ###########################################

    ######################################
    #|     ROUTER #1 ROUTING TABLE      |#
    #|----------------------------------|#
    #| Destination | First Hop | Metric |#
    #|----------------------------------|#
    #|      2      |    2      |    1   |#
    #|      3      |    3      |   16   |#
    #|      4      |    4      |    1   |#
    ######################################

    Router 3 is unresponsive for 24s

    ###########################################
    #|  ROUTER #1 NEIGHBOUR ROUTER(S) STATUS |#
    #|---------------------------------------|#
    #| ID | Port | Metric | Active | Garbage |#
    #|---------------------------------------|#
    #|  2 | 1025 |    1   |  True  |  False  |#
    #|  3 | 1035 |    3   | False  |   True  |#
    #|  4 | 9035 |    1   |  True  |  False  |#
    ###########################################

    ######################################
    #|     ROUTER #1 ROUTING TABLE      |#
    #|----------------------------------|#
    #| Destination | First Hop | Metric |#
    #|----------------------------------|#
    #|      2      |    2      |    1   |#
    #|      4      |    4      |    1   |#
    ######################################
    ```

    ### RIP (University of Canterbury) Project Collaborators/Contributors

    * [Liam Laing (now Vesper Haven)](https://profiles.canterbury.ac.nz/Vesper-Haven)
    * [Adam Ross](https://github.com/r055a)
  </details>
* <details><summary>Network Flow Optimisation</summary>

    ## Flow

    Python application for generating LP files used in printing [CPLEX](https://github.com/R055A/Flow/blob/master/doc/cplex.log) XYZ-network flow solutions, where X & Z are source & destination nodes, respectively; & Y are transit nodes, with constraints `X >= Z > 0`, `Y > 2`.

    ### Flow Example

    CPLEX output for generated LP input file `Y=7.lp`, where `X = Y = Z = 7`:

    ```text
    Problem 'Y=7.lp' read.
    Read time = 0.00 sec. (0.07 ticks)
    Tried aggregator 2 times.
    MIP Presolve eliminated 105 rows and 56 columns.
    MIP Presolve modified 47 coefficients.
    Aggregator did 392 substitutions.
    Reduced MIP has 56 rows, 344 columns, and 693 nonzeros.
    Reduced MIP has 343 binaries, 0 generals, 0 SOSs, and 0 indicators.
    Presolve time = 0.00 sec. (1.24 ticks)
    Found incumbent of value 130.666667 after 0.00 sec. (1.45 ticks)
    Probing time = 0.00 sec. (0.20 ticks)
    Cover probing fixed 0 vars, tightened 6 bounds.
    Tried aggregator 1 time.
    Reduced MIP has 56 rows, 344 columns, and 693 nonzeros.
    Reduced MIP has 343 binaries, 0 generals, 0 SOSs, and 0 indicators.
    Presolve time = 0.00 sec. (0.56 ticks)
    Probing time = 0.00 sec. (0.05 ticks)
    MIP emphasis: balance optimality and feasibility.
    MIP search method: dynamic search.
    Parallel mode: deterministic, using up to 8 threads.
    Root relaxation solution time = 0.00 sec. (0.58 ticks)

            Nodes                                         Cuts/
    Node  Left     Objective  IInf  Best Integer    Best Bound    ItCnt     Gap

    *     0+    0                          130.6667        4.6667            96.43%
          0     0       56.0000    10      130.6667       56.0000       92   57.14%
    *     0+    0                           58.0000       56.0000             3.45%
          0     0       56.0000    14       58.0000      Cuts: 14      101    3.45%
    *     0+    0                           57.6667       56.0000             2.89%
          0     0       56.0000    15       57.6667       Cuts: 7      112    2.89%
    *     0+    0                           57.3333       56.0000             2.33%
    *     0+    0                           57.0000       56.0000             1.75%
    *     0+    0                           56.3333       56.0000             0.59%
    *     0+    0                           56.0000       56.0000             0.00%
          0     0        cutoff             56.0000       56.0000      112    0.00%
    Elapsed time = 0.07 sec. (21.25 ticks, tree = 0.01 MB, solutions = 7)

    Root node processing (before b&c):
    Real time             =    0.07 sec. (21.32 ticks)
    Parallel b&c, 8 threads:
    Real time             =    0.00 sec. (0.00 ticks)
    Sync time (average)   =    0.00 sec.
    Wait time (average)   =    0.00 sec.
                            ------------
    Total (root+branch&cut) =    0.07 sec. (21.32 ticks)

    Solution pool: 7 solutions saved.

    MIP - Integer optimal solution:  Objective =  5.6000000000e+01
    Solution time =    0.07 sec.  Iterations = 112  Nodes = 0
    Deterministic time = 21.32 ticks  (306.31 ticks/sec)


    Incumbent solution
    Variable Name           Solution Value
    r                            56.000000
    xI1K1J1                       0.666667

    ...

    lK7                          56.000000
    All other variables in the range 1-792 are 0.
    ```

    ### Flow (University of Canterbury) Project Contributors

    * [Robert Loomes](https://github.com/robloomes)
    * [Adam Ross](https://github.com/r055a)
  </details>

## Requirements

- Node.js `v26.6.4`
- Rust stable

## Install

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

```bash
npm install
```

## Run

```bash
npm run dev:all
```

## Build

```bash
npm run build:all
```

# Contribute

Before making a Pull Request, ensure it addresses an Issue, and verify the branch passes:

```bash
npm run verify:fix
```
