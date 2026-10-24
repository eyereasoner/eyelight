% Keep conflicting animal observations as separate facts. For each property,
% the program distinguishes support for true, support for false, support for
% both, and no evidence either way.
%
% Conflicting or missing evidence leads to an undecided result rather than an
% arbitrary choice of truth value. Only an unconflicted result determines the
% movement decision. The treatment of contradictions is implemented by these
% ordinary rules.

bird(tweety).
penguin(tweety).
bird(falco).
penguin(opus).
mammal(batsy).
bat(batsy).
fish(nemo).
bird(mythic).
% An explicit observation that contradicts the rule for birds.
observed(mythic, flies, false).

% Domain rules that may conflict with one another.
flies(X, true) :- bird(X).
wings(X, true) :- bird(X).
flies(X, false) :- penguin(X).
wings(X, false) :- mammal(X).
flies(X, true) :- bat(X).
wings(X, true) :- bat(X).
swims(X, true) :- fish(X).
wings(X, false) :- fish(X).
flies(X, Value) :- observed(X, flies, Value).

% Local summaries. These inspect the rule conclusions, so they run only after
% the stratum above has reached its fixpoint.
flight_status(X, both) :- flies(X, true), flies(X, false).
flight_status(X, true_only) :- flies(X, true), \+ flies(X, false).
flight_status(X, false_only) :- flies(X, false), \+ flies(X, true).
wing_status(X, both) :- wings(X, true), wings(X, false).
wing_status(X, true_only) :- wings(X, true), \+ wings(X, false).
wing_status(X, false_only) :- wings(X, false), \+ wings(X, true).

% A conflicting summary is reported rather than silently resolved.
inconsistent(X, flies) :- flight_status(X, both).
needs_review(X, flies) :- flight_status(X, both).
inconsistent(X, wings) :- wing_status(X, both).
needs_review(X, wings) :- wing_status(X, both).

% Decisions read the summary, so a contradiction yields undecided instead of
% both answers at once.
flies_safely(X, true) :- flight_status(X, true_only).
flies_safely(X, false) :- flight_status(X, false_only).
flies_safely(X, undecided) :- flight_status(X, both).
wings_safely(X, true) :- wing_status(X, true_only).
wings_safely(X, false) :- wing_status(X, false_only).
wings_safely(X, undecided) :- wing_status(X, both).
moves_by(X, flying) :- flight_status(X, true_only).
migrates(X, true) :- flight_status(X, true_only).
moves_by(X, walking) :- flight_status(X, false_only).
migrates(X, false) :- flight_status(X, false_only).
moves_by(X, unknown) :- flight_status(X, both).
migrates(X, undecided) :- flight_status(X, both).

% Answer and Reason
?- flies_safely(Animal, Status).
?- wings_safely(Animal, Status).
?- why(wings_safely(batsy, undecided)).
?- needs_review(Animal, Property).
?- moves_by(Animal, Mode).

% Check
:- assertion(flies_safely(tweety, undecided)).
:- assertion(flies_safely(falco, true)).
:- assertion(flies_safely(opus, false)).
:- assertion(flies_safely(mythic, undecided)).
:- assertion(wings_safely(batsy, undecided)).
:- assertion(wings_safely(nemo, false)).
:- assertion(needs_review(tweety, flies)).
:- assertion(needs_review(mythic, flies)).
:- assertion(needs_review(batsy, wings)).
:- assertion(aggregate_all(count, needs_review(Animal, Property), 3)).
:- assertion(moves_by(tweety, unknown)).
:- assertion(moves_by(falco, flying)).
:- assertion(moves_by(opus, walking)).
:- assertion(\+ flies_safely(tweety, true)).
:- assertion(\+ flies_safely(tweety, false)).
