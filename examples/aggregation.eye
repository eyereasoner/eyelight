% Department budgets
%
% Each purchase belongs to a department and has a price. Eyelight counts
% purchases and computes each budget, including an empty department.
% Aggregate-local variables stay inside braces; item keys keep equal prices
% on different purchases distinct when summing.

department(library).
department(garden).
department(museum).
purchase(library, atlas, 20).
purchase(library, dictionary, 20).
purchase(garden, seeds, 5).

budget(Department, Items, Total) :-
    department(Department),
    count { Item : purchase(Department, Item, Price) } = Items,
    sum { Price, Item : purchase(Department, Item, Price) } = Total.

range(Department, Cheapest, Dearest) :-
    department(Department),
    min { Price : purchase(Department, Item, Price) } = Cheapest,
    max { Price : purchase(Department, Item, Price) } = Dearest.

?- budget(Department, Items, Total).
?- range(Department, Cheapest, Dearest).
