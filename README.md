# CS 110L Spring 2020 starter code

Assignment handouts are available [here](https://reberhardt.com/cs110l/spring-2020/).

Project 1: The DEET Debugger

Description: Build a simple debugger—likely using ptrace—that mirrors or extends features from CS 110. Designed for Linux environments (due to platform differences in debugging APIs). 
links.takashiidobe.com
reberhardt.com

Project 2: Balancebeam (Load Balancer)

Objective: Create a load balancer in Rust.

Key components:

Use httparse and http crates to parse and construct HTTP requests/responses via request.rs and response.rs.

Implement failover logic to handle crashed upstream servers.

Handle multi-threading, or optionally asynchronous operation using Tokio (async/await), to serve requests efficiently.

Includes performance benchmarks via “balancebench” to measure design choices.

Week2:
This week’s exercises will continue easing you into Rust. You’ll get some practice with handling ownership/references and working with Option and Result, and you’ll also get some light exposure to object-oriented programming in Rust! The primary exercise involves implementing a simple version of the diff utility to compare two files. Optionally, you can also implement the wc (word count) utility, or try out some graphics and implement Conway’s Game of Life.

Week3:
In the first part of these exercises, you’ll work through implementing a tool for inspecting file descriptors that you can use to debug your CS 110 assignments. This will give you more practice with structs and error handling.

The second part of the exercises will give you some experience with traits in Rust by implementing LinkedList.

Week5:
The goal of this week’s exercise is to get you thinking about multithreading material and to help you ask questions about lecture material that still feels confusing.

Week6:
In this week’s exercises, you’ll get to appreciate the sleekness of channels, a concurrency abstraction.

You want to share the joys of parallelism with your friends who haven’t learned about synchronization primitives yet by implementing for them a special, speedy function. This function takes two arguments: a vector of type Vec<T> another function f which takes elements of type T as input and returns type U as output. It runs f on each input element in the input vector and collects the results in an output vector. Even better, it does this in parallel! 