# How to

`All posibilities in example.json5`

## How to print

```json5
[
  "Just string in array", //equally println: "Just string in array"
  ["array", "of", "strings"], // equally println: ["in Function"]
  {
    print: ["in Function"],
  },
  {
    println: ["in Function"],
  },
]
```

## How to calclulate values

works only with numbers (and variables with number type)

```json5
[
  { calc: [2, "*", 3] }, //only 3 arguments
  { calc: [{ var: "some_variable" }, "-", 2] }, //{var:"some_var"} this is a way to get a variable
]
```

### Supported operators

1. \+
2. \-
3. \*
4. \/
5. \%
6. \>\>
7. \<\<
8. \^
9. \&
10. \|

## How to compare values

```json5
[
  { comp: [true, "!=", false] }, //only 3 arguments
  {
    comp: [
      {
        comp: [
          { comp: [{ calc: [1, "+", 1] }, ">", 3] },
          "==",
          { var: "var_with_bool_value" },
        ],
      },
      "&&",
      { comp: [{ comp: [{ calc: [1, "+", 1] }, ">", 3] }, "==", true] },
    ],
  }, //more complex comparisons: (( 1 + 1 > 3 ) == var_with_bool_value) && (( 1 + 1 > 3 ) == true)
]
```

### Supported operators for compare

1. ==
2. !=
3. \>
4. \<
5. \>=
6. \<=
7. \&\&
8. \|\|

## How to create a variable

```json5
[
  {
    let: {
      str: "A",
      num: 2,
      arr: ["Array", "in", "variable"],
    },
  },

  {
    let: {
      calculated: { calc: [{ var: "num" }, "*", 4] }, //result 8
    },
  },

  {
    let: {
      referenceVar: { ref: "calculated" },
    },
    //creates a reference variable, when the "calculated"
    // is changed, "referenceVar" will also be changed, in the
    // future it will be possible to change "referenceVar"
    // and the "calculated" will be changed
  },

  {
    let: {
      array: { arr: [{ var: "num" }, 4] }, //create calculated array result [2, 4]
    },
  },

  {
    let: {
      objVar: { obj: { var: "num" } }, //create object (hashmap) variable result {var: "num"}
    },
  },

  {
    let: {
      wrongObjVar: { var: "num" }, //result 2
    },
  },
]
```

## How to assign variable

```json5
[
  {
    assign: {
      calculated: { calc: [{ var: "calculated" }, "+", 1] }, // calculated = calculated + 1
    },
  },
]
```

## Loops

```json5
[
  {
    loop: [
      {
        if: {
          condition: { comp: [{ var: "i" }, ">=", 10] }, //if i >= 10 break loop
          body: ["break"],
          //else: [..commands] also work
        },
      },
      { assign: { i: { calc: [{ var: "i" }, "+", 1] } } }, // i += 1
      { print: ["\ri = ", { var: "i" }] },
      { sleep: 500 }, //sleep 500 ms
    ],
  },
]
```

## Input from console

```json5
[
  {
    let: {
      name: { input: "Your name: " },
    },
  },
  { print: ["Bye, ", { var: "name" }, "!"] },
]
```

## Delete variable

```json5
[
  {
    let: {
      name: { input: "Your name: " },
    },
  },
  { print: ["Bye, ", { var: "name" }, "!"] },
  { delete: "name" }, //deletes variable from memory
]
```

## Create scope

```json5
[
  {scope:[..commands]}
]
```
