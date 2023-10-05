# Golem Auction App

This is an example of an auction application implemented using Golem. The application uses two types of workers: an auction registry that maintains a list of all auctions and an auction worker for each auction that manages bidding for that auction.

By sharding the state this way the application can be highly scalable because users only need to interact with the single auction registry when creating new auctions or listing all auctions. Auction listing could also be cached.

Our auction will not require any database because we can just store the state of the auction registry and each auction with in memory data structures and rely on Golem's durability guarantees. Of course if we had large files like images to go along with our auctions we might still want to use a database.

To run an auction we first need to upload the templates for our auction worker and auction registry. Let's upload the template for the auction worker first since we will need to provide the template identifier for the auction worker to the auction registry so it can create new auction workers as needed.

The code for both of our workers are written in Rust, though we code have written them in any language that compiles to WebAssembly ("WASM"). To compile them to WASM we just do:

```bash
cargo component build --release
```

This will create two files, `auction.wasm` and `auction-registry.wasm`, in the `target/wasm32-wasi-release` folder that represent the code for our workers compiled to WASM. Copy them to the current directory so they are easily accessible.

We can upload the template for the auction worker to Golem like this:

```bash
golem-cli template add --template-name auction-1 auction.wasm
```

This will return some metadata about the template we have just uploaded:

```
template_id: 69dd184e-1fef-4925-800c-8a0d91ef2ef3
template_version: 0
template_name: auction-1
template_size: 2143417
exports:
- 'golem:template/api/initialize(auction: {auction-id: {auction-id: str}, name: str, description: str, limit-price: f32, expiration: u64}) => '
- 'golem:template/api/bid(bidder-id: {bidder-id: str}, price: f32) => variant(auction-expired: (), price-too-low: (), success: ())'
- 'golem:template/api/close-auction() => {bidder-id: str}?'
```

Of particular relevance to us is the `template_id`, which uniquely identifies each worker template. We will need that shortly when we create the auction registry worker so it knows how to create new auction workers as needed.

You will also notice that the metadata includes the signatures of all the functions that the worker exports. These exported functions represent the public API of our worker and we will use that later to bid on our auction.

We can upload the template for the auction registry for Golem in the same way:

```bash
golem-cli template add --template-name auction_registry-1 auction_registry.wasm
```

```
template_id: d4da0a79-a31c-43ca-be6c-2a7fddd9e33e
template_version: 0
template_name: auction-registry-1
template_size: 2710063
exports:
- 'golem:template/api/create-bidder(name: str, address: str) => {bidder-id: str}'
- 'golem:template/api/create-auction(name: str, description: str, limit-price: f32, expiration: u64) => {auction-id: str}'
- 'golem:template/api/get-auctions() => [{auction-id: {auction-id: str}, name: str, description: str, limit-price: f32, expiration: u64}]'
```

We're almost ready to create our first worker. We just need one more thing which is our authorization token, which the auction registry will need to have access to in order to create new auction workers on our behalf.

We can get it like this:

```bash
golem-cli token list
```

This token gives the ability to interact with Golem services on your behalf so be careful sharing it!

With this we have everything we need to deploy our auction service. We can create our auction registry work like this:

```bash
golem-cli worker add --template-name auction-registry-1 --worker-name auction-registry -1 --env "GOLEM_AUTHORIZATION_TOKEN"="********-****-****-****-************" --env "AUCTION_TEMPLATE_ID"="69dd184e-1fef-4925-800c-8a0d91ef2ef3"
```

```
workerId:
  rawTemplateId: d4da0a79-a31c-43ca-be6c-2a7fddd9e33e
  workerName: auction-registry-1
templateVersionUsed: 0
```

We are using environment variables to provide our auction registry worker with both our authorization token and the identifier for the template of our auction worker. Internally our auction registry worker will be able to use this information to create new auction workers for each new auction we create.

Since the auction registry will take care of creating new auctions as needed there is nothing else we need to do to deply our application.

Let's get started by registering as a bidder. We can see from the metadata for the auction registry that `create-bidder` expects two strings representing the bidder's name and address and returns a `bidder-id`:

```bash
golem-cli worker invoke-and-await --template-name=auction-registry-1 --worker-name=auction-registry-1 --function=golem:template/api/create-bidder --parameters='["Adam", "123 green street"]'
```

```
- bidder-id: a11ff221-d861-42e2-bc49-23b48b722ee3
```

Let's also create an auction. An auction requires an item, a description, a limit price, and an expiration date in seconds since the epoch.

We'll use an expiration date that correponds to October 12, 2023, about one week in the future as of when this was written. This should be updated to a date that is in the future as of when we create our auction so that we can bid on it.

```bash
golem-cli worker invoke-and-await --template-name=auction-registry-1 --worker-name=auction-registry-1 --function=golem:template/api/create-auction --parameters='["My first auction", "A simple auction", 100, 1697083549]'
```

```
- auction-id: 6fff4e1c-e7d6-49dc-b60c-2484ab6d7a4c
```

This `auction-id` is the key for us to interact with our auction directly.

Let's try bidding on our auction. We'll first try to enter a bid that is below the limit price:

```bash
golem-cli worker invoke-and-await --template-name=auction-1 --worker-name=auction-6fff4e1c-e7d6-49dc-b60c-2484ab6d7a4c --function=golem:template/api/bid --parameters='[{ "bidder-id": "a11ff221-d861-42e2-bc49-23b48b722ee3" }, 50]'
```

```
- price-too-low: null
```

Our price was too low! Let's try again with a higher price!

```bash
golem-cli worker invoke-and-await --template-name=auction-1 --worker-name=auction-6fff4e1c-e7d6-49dc-b60c-2484ab6d7a4c --function=golem:template/api/bid --parameters='[{ "bidder-id": "a11ff221-d861-42e2-bc49-23b48b722ee3" }, 200]'
```

```
- success: null
```

Our bid was successful!
