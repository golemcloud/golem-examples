package example

final case class State(userId: String, items: List[ProductItem]) { self =>
  def withUserId(userId: String): State = self.copy(userId = userId)

  def addItem(item: ProductItem): State = self.copy(items = self.items :+ item)

  def removeItem(productId: String): State = self.copy(items = self.items.filterNot(_.productId == productId))

  def updateItemQuantity(productId: String, quantity: Integer): State =
    self.copy(items = self.items.map { item =>
      if (item.productId == productId) ProductItem(item.productId, item.name, item.price, quantity)
      else item
    })

  def clear: State = self.copy(items = List.empty)
}
object State                                                     {
  val empty = State(userId = "", items = List.empty)
}

@cloud.golem.WitExport
object ComponentName extends Api { self =>
  private var state = State.empty

  def initializeCart(userId: String): WitResult[String, String] = {
    println(s"Initializing cart for user $userId")
    if (math.random() > 0.1) {
      state = state.withUserId(userId)
      WitResult.ok(userId)
    } else WitResult.err("Error while initializing cart")
  }

  def addItem(item: ProductItem): Unit = {
    println(s"Adding item to the cart of user ${state.userId}")
    state = state.addItem(item)
  }

  def removeItem(productId: String): Unit = {
    println(s"Removing item with product ID $productId from the cart of user ${state.userId}")
    state = state.removeItem(productId)
  }

  def updateItemQuantity(productId: String, quantity: Integer): Unit = {
    println(s"Updating quantity of item with product ID $productId to $quantity in the cart of user ${state.userId}")

    state = state.updateItemQuantity(productId, quantity)
  }

  def checkout(): CheckoutResult = {
    def reserveInventory(): Either[String, Unit] =
      if (math.random() < 0.1) Left("Inventory not available") else Right(())

    def chargeCreditCard(): Either[String, Unit] = Right(())

    def generateOrder(): String = "238738674"

    def dispatchOrder(): Either[String, Unit] = Right(())

    def clearState(): Unit = state = state.clear

    val result =
      for {
        _      <- reserveInventory()
        _      <- chargeCreditCard()
        orderId = generateOrder()
        _      <- dispatchOrder()
        _       = clearState()
        _       = println(s"Checkout for order $orderId")
      } yield OrderConfirmation(orderId)

    result match {
      case Right(orderConfirmation) => CheckoutResult.success(orderConfirmation)
      case Left(error)              => CheckoutResult.error(error)
    }
  }

  def getCartContents(): WitList[ProductItem] = {
    println(s"Getting cart contents for user ${state.userId}")
    WitList.fromList(state.items)
  }

  def getFirstItem(): WitOption[ProductItem] = {
    println(s"Getting first item for user ${state.userId}")
    WitOption.fromOption(state.items.headOption)
  }
}
