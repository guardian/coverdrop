### Create a pdf of the docs

## On MacOS

```
brew install pandoc basictex groff gs
# This refreshes your path, alternatively open a new terminal
eval "$(/usr/libexec/path_helper)"
cd docs
pandoc *.md -s -o coverdrop-docs.pdf --pdf-engine=pdfroff
```
