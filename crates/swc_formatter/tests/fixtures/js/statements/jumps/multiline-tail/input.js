label: while(a){ if(b) break label /* marker
 continued */ ; continue label; }
after: while(a){ if(b) break after; /* marker
 continued */ continue after; }
