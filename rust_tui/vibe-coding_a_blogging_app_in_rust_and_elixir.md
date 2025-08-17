# Vibe-coding a blogging app in Rust and Elixir

Writing is hard. Late one night, reflecting on how I'd effortlessly plopped the highlights of a hike, with photos, into a Slack thread, I had the punch-drunk inspiration to find out if a similar combination of constraints and conveniences could help with other kinds of writing.

Feeling good about Claude and Claude Code, I decided to put my pants on my head and make it a CLI (later upgraded to TUI), in Rust.

Further, because Parenthesis II app is an Elixir front end for state that lives in a Rust program, I liked the symmetry of this being a Rust "front end" with the data persisted by an Elixir app. Also, for the record: Claude assured me that this was a brilliant design.

I had some requirements in mind for the working product.

* it should work on Mac, because that's where I am. 
* the experience overall should be much like composing messages in Bluesky or Slack 
* entries would be limited in length and there'd be a character count display 
* it should be easy to attach an image to an entry 
* we're not learning Vim or Emacs keys

I didn't know if those were all doable.

In Claude Code's plan mode, I got a fairly involved design doc for a minimal version of the Rust bit, using the ratatui TUI library, and the Elixir bit for storing state in SQLite using Ecto. I got a slight API mismatch out of asking Claude to implement them separately, but since clearing that up, there's been basically nothing to say about the Elixir/Ecto/SQLite side. It's pretty simple and Claude basically one-shotted it.

Except, (particularly) if you're developing your app while populating your database, don't forget to back up anything you may want to keep that could get stomped on in a bad migration.


Prior to this, passing my eyeballs over some of Corrosion's source code was just about all my Rust experience, and I really didn't know where to start. Claude generated a small thing that did something, and the great thing then is that I suddenly had Rust code. I could start asking LLMs to explain it function by function, or line by line, and it wasn't a hello world; i actually cared what it did.

Interstitial thought: noisy logging may eat more tokens than needed, and pollute the context?


Some observations

* I wanted mouse interaction, particularly in the editor. 
* Claude struggles with counting the lines and columns in a Ratatui block. 
* You need an offset to account for the border and/or padding of the block.

_Implementing word wrap is hard. Complicated. Error-prone._  Don't do it if you don't have to (like if you can use an existing library).



## Using Rust

I wanted to touch Rust but not too much. How much would I learn, and how much would an LLM-based process insulate me from learning?

## Word wrapping

Of the things I took for granted from browser-based interfaces, and there were a few, soft-wrapping in a textarea is probably the biggest deal for me here. Stubbornness is why this is a TUI app, but stubbornness is also why I didn't want to compromise on wrapping OR mouse support.

I really wanted to try making something I can use. And to use it, I need to be thinking as little as possible about the mechanics of entering content.

There's no word wrapping in tui-textarea. It uses the Paragraph widget, which does have wrapping but doesn't give you access to the resulting lines (I think). Which are helpful for things like scrolling or arrow-keying through screen lines.

There's an existing editor crate with wrapping: edtui; but it's very invested in Vim, and while I did test the waters for adapting it, overriding that preference got complicated and I changed tack.



 If there were, it would mess with one of my other fussy requirements: using the mouse to edit text. We need the cursor to land on the visual position of the characters we aimed the mouse at. Ratatui's Paragraph widget does handle soft wrapping, but the mouse position doesn't know what character is where.s. In menus that use Paragraph, I decided just not to wrap.



I couldn't do without some wrapping in the text editor, though. I decided to try an adaptive hard wrap. This was very difficult with Claude, at least using the angles I tried. I had a hard time (read: failed, several times) keeping it from adding complications every time I interacted with it until I had to start over. Honestly, Claude has, or I gave it, weird ideas about how this seemingly simple (if repetitive) process should work.

Wrapping is pretty good but a WIP.

Writing is hard. This article alone took days of programming.

I built myself some constraints and some conveniences. I've been noodling on this post as I've gone along; word wrapping was far and away the biggest pit I fell into, and the app was pretty usable early on. But my mind was more urgently on the app than on the writing. Even now, I'm thinking about the rough edges.
 

![Image](images/threads/5c29294d-dd67-47e7-891e-6b454d80a960/e7cab497-a64a-47fc-98bc-bd9b2e351a88/hey.png)

Even if I've guessed the right combo of constraints and convenience, did I build in a fatal flaw in the very fact that I'm aware of imperfections and have the power to fix them? If I don't ever just use it, 

I have been able to live in an uneasy peace with bugs in my other homemade tools, so it's possible. It's an important 



## Elixir?

It's not entirely arbitrary. My blog runs on Elixir. There's no reason my Rust program couldn't own its own SQLite database, I can move a slider to put the boundary between the blog and the Rust app wherever I like. 

My initial reasoning had to do with the fact that I'm comfortable working with Phoenix and Ecto and would be better able to debug schema and db things there. As it turned out, Claude is comfy with them too, and I have hardly had to look at that side of the app.

I'm putting a screenshot in this so I can show off images.

![Image](images/threads/5c29294d-dd67-47e7-891e-6b454d80a960/screenie.png)

## Keybindings

We can't use Enter with  modifiers. You get around that in JS using `preventDefault`. Here, I'm just using Ctrl+s to save edits.

In iTerm2, to move the cursor one word left or right with alt+left/right through settings -> profiles -> keys -> key bindings -> presets -> natural text editing

In Ghostty this seems to be the default.


## Images

It was important to me that I be able to add images with very little effort. The truth is, with LLMs it'll be very easy to add a step that resizes or whatever if I can get the image loaded. But I want, straight away, to have some stupid-simple way to add an image to an entry. Ideally: paste from the OS clipboard.

I was having the thread view UI load each entry's image (if any) and size it to fit the widget space on demand. I'm trying to brute-force this, not get too fancy, but this was too slow.

I compromised: when you load a "thread", it loads, resizes (if needed), and caches all that thread's entries' images.

So loading the thread is noticeably laggy. But on my Macbook, switching between entries with previews or opening the editor with one is comfortable.

## Other compromises

* Can't select text in the vanilla Paragraph widget, so where I'm using that, you...can't select text. This isn't entirely surprising in an interface developed without the mouse in mind.

* Because of the circuitous route I took to composing this, there'll be inefficiencies, inconsistencies, and magic numbers.

