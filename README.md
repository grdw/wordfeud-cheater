# Wordfeud cheater

A little ditty that can figure out what the best words are to put to the table.

## Setup:
### Step 1: Setting up your language.
#### Language: Dutch
1. Download the latest wordlist from [Opentaal](https://github.com/OpenTaal/opentaal-wordlist)
2. Add it under `data/nl/wordlist.txt`

### Step 2: Setting up your current board
Do `cp example.board current.board` and fill out the current.board file with your letters. Every dot is a single letter.

### Step 3 (Optional): Changing board layout
The default board layout is what is used under `default-layout.board`. If you need to use a different layout, you can. A `.` indicates a non-bonus space, and the numbers indicate the following:

1. Start tile
2. Double letter
3. Triple letter
4. Double word
5. Triple word

### Step 4: Run the code:

```bash
cargo run nl "ABCDEFG"
```

Where `ABCDEFG` are the letters you currently have.
