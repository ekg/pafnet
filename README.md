# pafnet

PAF (pairwise alignment format) to [network/graph](https://gephi.org/users/supported-graph-formats/) format converter.
This tool is designed to project PAF alignments (the implied overlap and containment graph) into a network format that can be processed by tools like [grappolo](https://github.com/Exa-Graph/grappolo) and [gephi](https://gephi.org/)

## usage

For instance, to convert into Pajek NET format:

```
pafnet -n x.paf >x.net
```

Or to write an edgelist:

```
pafnet -e x.paf >x.edges
```

## building

Build using cargo. This is rust :)
