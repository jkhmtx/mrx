This repository was cloned from [izharishaksa/ecommerce-system-example](https://github.com/izharishaksa/ecommerce-system-example/commit/cab109e04712f6770a1b6acddfdcb30215259dc3), with thanks to `izharishaksa`.

This sub-project is individually licensed from `mrx`, using the original license present in the origin repository.

The original README continues below.

---

# Simple Ecommerce System

Project page: https://github.com/users/izharishaksa/projects/6/views/1

Implementing Event Driven Architecture and use Domain Driven Design approach.

[![Customer Service](https://github.com/izharishaksa/ecommerce-system-example/actions/workflows/customer-service.yaml/badge.svg)](https://github.com/izharishaksa/ecommerce-system-example/actions/workflows/customer-service.yaml)
[![Inventory Service](https://github.com/izharishaksa/ecommerce-system-example/actions/workflows/inventory-service.yaml/badge.svg)](https://github.com/izharishaksa/ecommerce-system-example/actions/workflows/inventory-service.yaml)
[![Order Service](https://github.com/izharishaksa/ecommerce-system-example/actions/workflows/order-service.yaml/badge.svg)](https://github.com/izharishaksa/ecommerce-system-example/actions/workflows/order-service.yaml)

## Run instructions

1. Run `docker-compose up`
2. Please wait until all services running
3. Demonstrate using `ecommerce-system-example.postman_collection.json`

## Test Case Scenario

1. Create product `POST /products`
2. Register customer `POST /customers`
3. Create order `POST /orders`, order status is `placed`, event `ORDER_PLACED` is sent
4. `ORDER_PLACED` is consumed by `inventory-service`, if inventory is enough or product is exist, event `ORDER_CREATED` is sent, otherwise `ORDER_REJECTED`. Stock and sold are updated accordingly.
5. `ORDER_CREATED` is consumed by `order-service`, status and total price is updated
6. `ORDER_REJECTED` is consumed by `order-service`, status is updated
