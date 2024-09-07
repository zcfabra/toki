# FunPy: If a Python ate a crab 

So it's Python, but with none of the annoying stuff (exceptions, classes, etc)
and many of the nicest features from Rust.

Notable Features:
- [ ] Expression-based (ifs, matches, etc are expressions)
```
a = (
    if condition: 10
    else: 20 
);

b = (
    match some_enum:
        case A: some_value
        case B: some_other_value
        case C: yet_another_value
);
```
- [ ] Immutable by default, with opt-in mutability 
```
a: mut int = 10;
a += 1;
```

- [ ] Memory-compact structs instead of sparse Python objects
```
struct Person:
    id: UUID
    name: str
    age: mut int

    def birthday_handler(self) -> ():
        self.age += 1
```
- [ ] Null safety via a Rust-style Option type
```
maybe_name = get_name_opt();

match maybe_name:
    case Some(name):
        print(name)
    case None:
        print("No Name Found")
```
- [ ] Errors-as-values through Rust-style Result types
```
def save_div(a: float, b: float) -> Result[float, DivByZeroErr]:
    if b == 0:
        Err(DivByZeroErr)
    else:
        Ok(a / b)
```
- [ ] Support for reference and value semantics
```
def fn_that_takes_a_list_ref(l: ref mut list[int]) -> ():
    l.append(40)

def fn_that_takes_a_list(l: mut list[int]) -> ():
    # This 'l' variable is local to the function, will not change the passed-in-list 
    # in the parent scope
    l.append(10)

def list_printer(l: mut list[int]) -> ():
    # Even thought this list is passed-by-value, no copy occurs
    # because Funpy implements 'copy-on-write'

    for each in l:
        print(each)

l: mut list[int] = [10, 20, 30] 

fn_that_takes_a_list_ref(ref l);

fn_that_takes_a_list(l);
list_printer(l);

print(l)  # '[10, 20, 30, 40]'

```
- [ ] Algebraic Data Types 
```
struct Lunch:
    cost: int
    type: LunchType

enum LunchType:
    Soup(volume: float)
    Sandwich(weight: float, toppings: ref list[str])
    Salad

enum MealType:
    Breakfast
    Lunch(Lunch)
    Dinner

```
- [ ] Type inference
```
# Both valid
a: int = 10;
b = 10;
```

