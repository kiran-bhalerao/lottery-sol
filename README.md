<!-- PROJECT LOGO -->
<br />
<p align="center">
    <img src="./solana-logo.png" alt="Logo" width="80" height="80">

  <h3 align="center">Sol Lottery</h3>

  <p align="center">
    Lottery program on solana blockchain
    <br />
    <a href="https://soprox.descartes.network/"><strong>Made with Soprox »</strong></a>
    <br />
    <br />
    
  </p>

</p>

## How it works

- Anyone can organize the Lottery program and advertize so people can participate in the Lottery.
- Lottery organizer gets commission for organizing lottery from the winner's lottery amount.
- Lottery organizer will decide whats the entry_fees and commission_rate at the time Lottery initialization.
- After Lottery init participants can participate by providing entry_fees (program will deduce entry fees from participants account).
- Finally Lottery organizer calls lottery program with pickWinner instruction then the Lottery program will choose one winner from n number of participants and transfer all pooled amount (minus commission) to winner.
