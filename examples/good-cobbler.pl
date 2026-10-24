% Extract a person's trade from a structured description such as [good,
% cobbler]. Joe is good at cobbling and Jane at carpentry in the supplied
% data. Keeping the trade in the description avoids treating "good" as a
% property that applies to every activity the person undertakes.

description(joe, [good, cobbler]).
description(jane, [good, carpenter]).
description(sam, [novice, cobbler]).

good_at(Person, Trade) :-
    description(Person, [good, Trade]).
classified_as(Person, Trade) :-
    description(Person, [good, Trade]).

% Answer and Reason
?- good_at(Person, Trade).
?- why.
?- classified_as(Person, Trade).
?- why.

% Check
:- assertion(good_at(joe, cobbler)).
:- assertion(good_at(jane, carpenter)).
:- assertion(classified_as(joe, cobbler)).
:- assertion(classified_as(jane, carpenter)).
:- assertion(\+ good_at(joe, carpenter)).
:- assertion(\+ good_at(sam, cobbler)).
:- assertion(\+ classified_as(sam, cobbler)).
:- assertion(aggregate_all(count, good_at(Person, Trade), 2)).
:- assertion(aggregate_all(count, classified_as(Person, Trade), 2)).
