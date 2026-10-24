% Finding paths through a network
%
% Direct links connect four places, with a cycle between two of them.
% Whye finds every place reachable from the library. Recursion stops
% when no new fact remains; revisiting a place does not duplicate an answer.

link(library, park).
link(park, station).
link(station, park).
link(station, museum).

path(From, To) :- link(From, To).
path(From, To) :- path(From, Via), link(Via, To).

?- path(library, Where).
