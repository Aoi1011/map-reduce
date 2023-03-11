# Distributed Systems

## Map Reduce

### Abstract
- for processing and generating large data sets
- parallelized and executed on a large cluster of commodity machines

### Programming model
2 main functions

1. Map
	written by the user, takes an input pair and produces a set of intermediate key/value pairs

2. Reduce
	also written by the user, accepts an intermediate key I and a set of values for that key. It merges togetherthese values to form a possibly smaller set of values. 

```
map(String key, String value): 
 // key: document name
 // value: document contents
 for each word w in value:
  EmitIntermediate(w, "1");

reduce(String key, Iterator values):
 // key: a word
 // values: a list of counts
 int result = 0;
 for each v in values:
  result += ParseInt(v);
 Emit(AsString(result));
```

### Implementation


### REFERENCES
- [MapReduce: Simplified Data Processing on Large Clusters](http://nil.csail.mit.edu/6.824/2020/papers/mapreduce.pdf)
- [6.824 Lab 1: MapReduce](http://nil.csail.mit.edu/6.824/2020/labs/lab-mr.html)
- [MapReduce - Computerphile](https://www.youtube.com/watch?v=cQP8WApzIQQ)
- [Lecture 1: Introduction](https://www.youtube.com/watch?v=cQP8WApzIQQ)
