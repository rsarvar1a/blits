
# The LITS Text Protocol

A standard for LITS notation across clients and engines.

# Notation 

The rules for notating LITS positions.

## Tiles 

### Individual Representations

The following describes the notation for each individual component of a tile.

```
player X: 'x' | 'X'
player O: 'o' | 'O'
```

```
colour L: 'l' | 'L'
colour I: 'i' | 'I'
colour T: 't' | 'T'
colour S: 's' | 'S'
```

```
blank   : '-' | '_'
```

### Tile State 

The overall state of a tile is an index representation of a (player, colour) tuple, where:

```
(p, c)  : 5 * as_index(p) + as_index(c)
```

## Tetrominoes

### Coordinate 

A coordinate of the form (i, j) notates to 'ij'.

Example:

```
(4, 5)
```

notates to 

```
45 
```

### Tetromino 

The overall notation for a tetromino is the tetromino colour followed by a list of coordinates.

Example:

```
L L L - - - - - - -
L - - - - - - - - -
- - - - - - - - - -
- - - - - - - - - -
- - - - - - - - - -
- - - - - - - - - -
- - - - - - - - - -
- - - - - - - - - -
- - - - - - - - - -
- - - - - - - - - -
```

notates to 

```
L[00,01,02,10]
```

## Board Views 

### Board Position

The board state is a 105-character hashstring corresponding to its tilestates in (r, c) 
order followed by single-digit counts for the number of each piece type remaining.

Example:

```
-L xL -L -- o- -- -- -- -- --
-L -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --

L remaining: 4 
I remaining: 5
T remaining: 5
S remaining: 5
```

notates to 

```
1610a00000 
1000000000 
0000000000 
0000000000 
0000000000 
0000000000 
0000000000
0000000000
0000000000
0000000000
,4555
```

(newlines added for clarity).

## Game and History 

The game state consists of a base board (the turn-0 setup position) 
followed by each tetromino placed in the move history on sequential 
lines.

Example:

```
-- x- -- -- o- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --

-L xL -L -- o- -- -- -- -- --
-L -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
-- -- -- -- -- -- -- -- -- --
```

notates to 

```
050a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,4555
L[00,01,02,10]
```

# Commands 

A minimal set of commands that must be supported by a LITS text protocol engine.

```
"cancel-search"  : Cancels an ongoing search request.

"gen-move"       : Requests that the engine find the best move in this position.
  param <player>      which player to optimize for 

"initialize"     : Initializes the backing engine.

"new-game"       : Starts a blank new game.

"play-move"      : Plays the given move into the current position.
  param <piece>       the notation of a tetromino 

"setup-position" : Starts a new game with the given board position. 
  param <board>       the hashstring of a board position

"shutdown"       : Halts the backing engine.

"undo-move"      : Rewinds the position to the previous move, if possible.
```
