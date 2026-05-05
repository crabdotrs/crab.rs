# Cookie Shop API

A complete REST API example demonstrating the Crab programming language's OOP features and Actix-web integration.

## Overview

This project is a capstone tutorial demonstrating production-ready patterns in Crab:

- Full OOP with encapsulation, inheritance, and polymorphism
- Design patterns: Factory, Builder, Repository, Strategy
- Async/await with Actix-web
- C interoperability with CBlock
- Comprehensive unit tests

## Running

```bash
crab run
```

The server starts at `http://127.0.0.1:8080`

## API Endpoints

### General

- `GET /` - API information
- `GET /health` - Health check

### Cookies

- `GET /cookies` - List all cookies
- `GET /cookies/{id}` - Get a specific cookie
- `POST /cookies` - Create a new cookie
- `POST /cookies/{id}/restock` - Restock a cookie
- `POST /cookies/{id}/purchase` - Purchase a cookie

### Customers

- `GET /customers` - List all customers
- `GET /customers/{id}` - Get customer by ID
- `POST /customers` - Create new customer
- `POST /customers/{id}/upgrade` - Upgrade to premium
- `GET /customers/premium` - List premium customers

### Orders

- `GET /orders` - List all orders
- `GET /orders/{id}` - Get order by ID
- `POST /orders` - Create new order
- `POST /orders/{id}/items` - Add item to order
- `POST /orders/{id}/confirm` - Confirm order
- `POST /orders/{id}/ship` - Ship order
- `POST /orders/{id}/cancel` - Cancel order
- `GET /orders/{id}/total` - Get order total
- `GET /revenue` - Get total revenue

## OOP Patterns Demonstrated

### Encapsulation

`src/models/cookie.crab` - Private fields with underscore prefix, public getters/setters

### Inheritance

`src/models/customer.crab` - PremiumCustomer extends Customer with override for discountRate

### Polymorphism

`src/models/customer.crab` - Repository<T> interface with multiple implementations

### Abstraction

`src/repositories/repository.crab` - Abstract class defining contract for all repositories

### Factory Pattern

`src/utils/factory.crab` - CookieFactory for creating different cookie types

### Builder Pattern

`src/utils/builder.crab` - OrderBuilder and CustomerBuilder for complex object construction

### Repository Pattern

`src/repositories/` - Abstract Repository<T> with Memory implementations

### Sealed Classes

`src/models/order.crab` - OrderStatus with exhaustive pattern matching

### C Interop

`src/utils/hash.crab` - CBlock with inline C hash functions

## Architecture

```
src/
  models/       - Data entities (Cookie, Order, Customer)
  repositories/ - Data access layer (Repository pattern)
  services/     - Business logic (DI pattern)
  routes/       - HTTP handlers (Actix-Web)
  utils/        - Helpers (Factory, Builder, C Interop)
tests/          - Unit tests demonstrating all patterns
```

## Testing

```bash
crab test
```

Runs unit tests for all OOP patterns including encapsulation, inheritance, polymorphism, and design patterns.

## Project Structure

This is a reference implementation showing production-ready Crab code with:

- Proper error handling with Result<T,E>
- Async/await throughout
- Type-safe dependency injection
- Comprehensive unit tests
