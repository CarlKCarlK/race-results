# `race-results`

An open-source project to create a tool to find your friends, club members, and teammates in race results.

You can run it live from [this GitHub-hosted page](https://carlkcarlk.github.io/race-results/matcher/).

## Privacy

The program runs entirely in your browser. No information leaves your computer.

## License

This is a dual-licensed open-source project.
Apache License, Version 2.0, or MIT license.

## Discussion

[race-results discussion](https://github.com/CarlKCarlK/race-results/discussions) on GitHub.

## How does it work?

You paste in a list of your club members and the race results of interest. It displays a list of the most likely matches. The program understands nicknames.

Under the covers, the program searches the results for the names and city of your club members. It assigns points for each match and subtracts different points for each miss. The points are based on the distinctiveness of the name, so matching "Chellie" is worth more than matching "Robert". The program assigns and maninulates points based on "naive Bayes" and probability theory.

Embedded inside the program is a table of 250,000 names and their distinctiveness. The program also includes a table of nicknames. The program is written in Rust and compiled "WASM" which runs in web browsers.

For an article about the method used, see: [Use Bayes’ Theorem to Find Distinctive Names in a List](https://medium.com/towards-data-science/use-bayes-theorem-to-find-distinctive-names-in-a-list-5acd8fe03c2b), *Towards Data Science*, Carl Kadie, 2021.
