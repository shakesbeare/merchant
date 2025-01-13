# About

This project uses csv data collected from [Archives of
Nethys](https://2e.aonprd.com/) to randomly generate an inventory for merchants
for use in your games. 

Currently, it limits items to appear based on the merchant level. It uses the
player "Treasure By Level" table (with a modifier) to produce inventories of an
appropriate wealth for that merchant's level. It also, at the moment, forces a 
certain amount of rations to appear.

# Usage

```
merchant help
```

# Embedding the lib into another project

```toml
# cargo automatically looks up the lib by name when specifying a git dependency
merchant_gen_lib = { git "https://github.com/shakesbeare/merchant" }
```


