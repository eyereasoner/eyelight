% A shop's price calculator
%
% Each product has a unit price and an ordered quantity. Whye computes
% the line totals and selects orders worth at least 20 units of currency.
% Explicit @ built-ins act as interpreted relations with bound inputs.

order(book, 12, 2).
order(pen, 3, 4).

total(Item, Total) :- order(Item, Price, Quantity), @mul(Price, Quantity, Total).
large_order(Item, Total) :- total(Item, Total), @ge(Total, 20).

?- total(Item, Total).
?- large_order(Item, Total).
?- @concat("Hello, ", "world", Greeting), @strlen(Greeting, Length).
