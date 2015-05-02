// File: TupleBST.cc
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Class definition for TupleBST
#include "stdinc.h"
#include "TupleBST.h"

TupleBST::TupleBST() {
  root = NULL;
  N = 0;
  PtrIndex = 0;
}

TupleBST::~TupleBST() {
  if (root != NULL) cleanup(root);
  delete(ListOfFilterIndexPtrs);
}

void TupleBST::cleanup(TupleBST_item* node){
  struct TupleBST_item *left;
  struct TupleBST_item *right;
  left = node->left;
  right = node->right;
  if (left != NULL) cleanup(left);
  if (right != NULL) cleanup(right);
  delete(node->FilterIndexListPtr);
  delete(node);
  return;
}

int TupleBST::scope(FiveTuple* ftuple){
  double scope5d;
  scope5d = (32 - ftuple->sa_len) + (32 - ftuple->da_len) + (log(ftuple->sp_wid)/log(2)) + (log(ftuple->dp_wid)/log(2)) + (8*(1 - ftuple->prot)) + (1 - ftuple->flag);
  return (int)scope5d;
}

int TupleBST::size(){return N;}

dlist* TupleBST::Insert(FiveTuple* ftuple){
  // printf("TupleBST::Insert: computing scope\n");
  int fscope = scope(ftuple);
  // printf("TupleBST::Insert: fscope = %d\n",fscope);
  TupleBST_item* y = NULL;
  TupleBST_item* x = root;
  // Search for tuple in tree
  while (x != NULL){
    // printf("TupleBST::Insert: x is not NULL\n");
    // printf("TupleBST::Insert: x->scope = %d\n",x->scope);
    y = x;
    if ((fscope == x->scope) && (ftuple->sa_len == x->tuple.sa_len) &&
	(ftuple->da_len == x->tuple.da_len) && (ftuple->sp_wid == x->tuple.sp_wid) &&
	(ftuple->dp_wid == x->tuple.dp_wid) && (ftuple->prot == x->tuple.prot) &&
	(ftuple->flag == x->tuple.flag)){
      // printf("TupleBST::Insert: tuple exists\n");
      return x->FilterIndexListPtr;
    }
    if (fscope < x->scope) x = x->left;
    else x = x->right;
  }
  // printf("TupleBST::Insert: creating new tuple\n");
  // Create new tuple
  TupleBST_item* z = new TupleBST_item;
  z->scope = fscope;
  z->tuple.sa_len = ftuple->sa_len;
  z->tuple.da_len = ftuple->da_len;
  z->tuple.sp_wid = ftuple->sp_wid;
  z->tuple.dp_wid = ftuple->dp_wid;
  z->tuple.prot   = ftuple->prot;
  z->tuple.flag   = ftuple->flag;
  z->parent = y;
  z->left = NULL;
  z->right = NULL;
  if (y == NULL) root = z;
  else {
    if (fscope < y->scope) y->left = z;
    else y->right = z;
  }
  z->FilterIndexListPtr = new dlist;
  N++;
  return z->FilterIndexListPtr;
}

dlist** TupleBST::GetTupleLists(){
  ListOfFilterIndexPtrs = new dlist*[N];
  // printf("TupleBST::GetTupleLists: created ListOfFilterIndexPtrs\n");
  InorderTreeWalk(root);
  // printf("TupleBST::GetTupleLists: done with InorderTreeWalk\n");
  return ListOfFilterIndexPtrs;
}

void TupleBST::InorderTreeWalk(TupleBST_item* node){
  if (node != NULL){
    InorderTreeWalk(node->left);
    // printf("TupleBST::InorderTreeWalk: Adding pointer to ListOfFilterIndexPtrs, N = %d, PtrIndex = %d\n",N,PtrIndex);
    ListOfFilterIndexPtrs[PtrIndex++] = node->FilterIndexListPtr;
    InorderTreeWalk(node->right);
  }
  return;
}

void TupleBST::PrintTree(){
  PrintNode(root);
  return;
}


void TupleBST::PrintNode(TupleBST_item* node){
  if (node == NULL) return;
  printf("Parent: ");
  if (node->parent != NULL)
    printf("[%d,%d,%d,%d,%d,%d]",node->parent->tuple.sa_len,node->parent->tuple.da_len,
	   node->parent->tuple.sp_wid,node->parent->tuple.dp_wid,node->parent->tuple.prot,node->parent->tuple.flag);
  printf("\n");

  printf("scope: "); printf("%d\n",node->scope);
  printf("Tuple: ");
  printf("[%d,%d,%d,%d,%d,%d]",node->tuple.sa_len,node->tuple.da_len,
	 node->tuple.sp_wid,node->tuple.dp_wid,node->tuple.prot,node->tuple.flag);
  printf("\n");

  printf("Left: ");
  if (node->left != NULL)
    printf("[%d,%d,%d,%d,%d,%d]",node->left->tuple.sa_len,node->left->tuple.da_len,
	   node->left->tuple.sp_wid,node->left->tuple.dp_wid,node->left->tuple.prot,node->left->tuple.flag);
  printf("\n");
  printf("Right: ");
  if (node->right != NULL)
    printf("[%d,%d,%d,%d,%d,%d]",node->right->tuple.sa_len,node->right->tuple.da_len,
	   node->right->tuple.sp_wid,node->right->tuple.dp_wid,node->right->tuple.prot,node->right->tuple.flag);
  printf("\n");
  
  // Recursive call
  PrintNode(node->left);
  PrintNode(node->right);

  return;
}

