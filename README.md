# taskz

a minimalistic todo list app written in rust.

## features

it's a basic to do list app but it features intelligent fuzzy matching when marking tasks as done or editing, allowing you to use approximate or partial phrases instead of exact matches. it finds the most similar task using levenshtein distance, making task management effortless even with typos or vague descriptions.

## installation

- **windows:**  
  run `taskz -i` as administrator (copies binary to system32)

- **linux:**  
  run `sudo taskz -i` (copies binary to /usr/local/bin)

## usage

- **add a task:**  
  `taskz add <task description>`

- **list tasks:**  
  `taskz list`  
  (add `-a` flag for alphabetical order)

- **search tasks:**  
  `taskz search <query>`

- **edit a task:**  
  `taskz edit <old description> /// <new description>`

- **mark task as done:**  
  `taskz done <task description>`

- **undo last removal:**  
  `taskz undo`

- **clear all tasks:**  
  `taskz clear`

- **help:**  
  `taskz -h` or `taskz /?` or `taskz -?`

## example

```sh

C:\>taskz add make a cool rap song
task added

C:\>taskz add talk to my cat
task added

C:\>taskz add pet my hamster
task added

C:\>taskz list
[1742388949] make a cool rap song
[1742388954] talk to my cat
[1742388971] pet my hamster

C:\>taskz edit talk to the cat /// feed my cat
task updated to: feed my cat

C:\>taskz done petting hamster
task done and removed: pet my hamster

C:\>taskz list
[1742388949] make a cool rap song
[1742388954] feed my cat

C:\>taskz add make a disstrack song on my hamster
task added

C:\>taskz search song
[1742388949] make a cool rap song
[1742389049] make a disstrack song on my hamster

C:\>taskz done rap song
task done and removed: feed my cat

C:\>taskz undo
undo successful: task restored

C:\>taskz list
[1742388949] make a cool rap song
[1742388954] feed my cat
[1742389049] make a disstrack song on my hamster

```

### copyright <C> made by traian/tra1an/ravegoth
