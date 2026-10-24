% Count each listed owner's dogs and apply the example's licensing rule: an
% owner needs a license when the count is greater than four. Alice has five
% dogs, Carol has exactly four, and Dana has none. The threshold is supplied
% by this program rather than taken from a jurisdiction's licensing rules.

owns(alice, dog1).
owns(alice, dog2).
owns(alice, dog3).
owns(alice, dog4).
owns(alice, dog5).
owns(bob, dog6).
owns(bob, dog7).
owns(carol, dog8).
owns(carol, dog9).
owns(carol, dog10).
owns(carol, dog11).
owner(alice).
owner(bob).
owner(carol).
owner(dana).

dog_count(Owner, N) :-
    owner(Owner),
    aggregate_all(count, owns(Owner, Dog), N).
requires(Owner, dog_license) :-
    dog_count(Owner, N),
    N > 4.

% Answer and Reason
?- dog_count(Owner, Count).
?- why.
?- requires(Owner, dog_license).
?- why.

% Check
:- assertion(dog_count(alice, 5)).
:- assertion(dog_count(bob, 2)).
:- assertion(dog_count(carol, 4)).
:- assertion(dog_count(dana, 0)).
:- assertion(requires(alice, dog_license)).
:- assertion(\+ requires(bob, dog_license)).
:- assertion(\+ requires(carol, dog_license)).
:- assertion(\+ requires(dana, dog_license)).
:- assertion(aggregate_all(count, requires(Owner, dog_license), 1)).
