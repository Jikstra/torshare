# TorShare


A CLI tool allowing the user to share files anonymously over the tor network. A similiar and more production ready solution is [OnionShare](https://onionshare.org/). Currently this project is a proof-of-concept and not recommended in productional use, especially not when the data is important/anonymity is not only optional but needed.

## Usage

To share a file you can simply run following command:
```
torshare share ~/my-secret-files/grandmas-cheese-cake-recipe.pdf
```

This command will start tor with a hidden service, a webserver that is accessable over the generated hidden service and serve the specified url under a specific, random url.

The output of this command will look similiar to this:
```
Sharing now!
        torshare download tklj4oyf4bcgcn4gwyhlvtb5pggtzw2cyihfymcetxhsdykhdfebxqyd.onion/ZRqysiim0jpL5TVdQ8yOT2bQE0ZVlj

Serving file ~/my-secret-files/grandmas-cheese-cake-recipe.pdf under /ZRqysiim0jpL5TVdQ8yOT2bQE0ZV
```

To download this file, you can now either use the tor browser bundle and open the url `http://tklj4oyf4bcgcn4gwyhlvtb5pggtzw2cyihfymcetxhsdykhdfebxqyd.onion/ZRqysiim0jpL5TVdQ8yOT2bQE0ZVlj` or execute following command:

```
torshare download tklj4oyf4bcgcn4gwyhlvtb5pggtzw2cyihfymcetxhsdykhdfebxqyd.onion/ZRqysiim0jpL5TVdQ8yOT2bQE0ZVlj
```

This will download the file to the current folder.

