% Places outside a travel network
%
% A traveler starts at the library and follows direct or indirect links.
% Eyelight first completes the recursive reachability relation, then finds
% places outside it. Stratified negation tests a completed lower stratum.

place(park).
place(station).
place(island).
link(library, park).
link(park, station).
link(station, park).

reachable(To) :- link(library, To).
reachable(To) :- reachable(Via), link(Via, To).
outside(Place) :- place(Place), not reachable(Place).

?- outside(Place).
