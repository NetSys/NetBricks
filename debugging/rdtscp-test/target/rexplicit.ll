; ModuleID = 'rdtscp_test.0.rs'
target datalayout = "e-m:e-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

%str_slice = type { i8*, i64 }
%"2.std::fmt::Arguments" = type { { %str_slice*, i64 }, %"2.std::option::Option<&'static [std::fmt::rt::v1::Argument]>", { %"2.std::fmt::ArgumentV1"*, i64 } }
%"2.std::option::Option<&'static [std::fmt::rt::v1::Argument]>" = type { { %"2.std::fmt::rt::v1::Argument"*, i64 } }
%"2.std::fmt::rt::v1::Argument" = type { %"2.std::fmt::rt::v1::Position", %"2.std::fmt::rt::v1::FormatSpec" }
%"2.std::fmt::rt::v1::Position" = type { i64, [0 x i64], [1 x i64] }
%"2.std::fmt::rt::v1::FormatSpec" = type { i32, i8, i32, %"2.std::fmt::rt::v1::Count", %"2.std::fmt::rt::v1::Count" }
%"2.std::fmt::rt::v1::Count" = type { i64, [0 x i64], [1 x i64] }
%"2.std::fmt::ArgumentV1" = type { %"2.core::fmt::Void"*, i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)* }
%"2.core::fmt::Void" = type {}
%"2.std::fmt::Formatter" = type { i32, i32, i8, %"2.std::option::Option<usize>", %"2.std::option::Option<usize>", { i8*, void (i8*)** }, %"2.std::slice::Iter<std::fmt::ArgumentV1<'static>>", { %"2.std::fmt::ArgumentV1"*, i64 } }
%"2.std::option::Option<usize>" = type { i64, [0 x i64], [1 x i64] }
%"2.std::slice::Iter<std::fmt::ArgumentV1<'static>>" = type { %"2.std::fmt::ArgumentV1"*, %"2.std::fmt::ArgumentV1"*, %"2.std::marker::PhantomData<&'static std::fmt::ArgumentV1<'static>>" }
%"2.std::marker::PhantomData<&'static std::fmt::ArgumentV1<'static>>" = type {}

@__rustc_debug_gdb_scripts_section__ = linkonce_odr unnamed_addr constant [34 x i8] c"\01gdb_load_rust_pretty_printers.py\00", section ".debug_gdb_scripts", align 1
@str4212 = internal constant [2 x i8] c"a "
@str4213 = internal constant [3 x i8] c" b "
@str4214 = internal constant [3 x i8] c" c "
@str4215 = internal constant [3 x i8] c" d "
@str4216 = internal constant [3 x i8] c" e "
@str4217 = internal constant [3 x i8] c" f "
@str4218 = internal constant [3 x i8] c" g "
@str4219 = internal constant [3 x i8] c" h "
@str4220 = internal constant [3 x i8] c" i "
@str4221 = internal constant [1 x i8] c"\0A"
@ref4222 = internal unnamed_addr constant [10 x %str_slice] [%str_slice { i8* getelementptr inbounds ([2 x i8], [2 x i8]* @str4212, i32 0, i32 0), i64 2 }, %str_slice { i8* getelementptr inbounds ([3 x i8], [3 x i8]* @str4213, i32 0, i32 0), i64 3 }, %str_slice { i8* getelementptr inbounds ([3 x i8], [3 x i8]* @str4214, i32 0, i32 0), i64 3 }, %str_slice { i8* getelementptr inbounds ([3 x i8], [3 x i8]* @str4215, i32 0, i32 0), i64 3 }, %str_slice { i8* getelementptr inbounds ([3 x i8], [3 x i8]* @str4216, i32 0, i32 0), i64 3 }, %str_slice { i8* getelementptr inbounds ([3 x i8], [3 x i8]* @str4217, i32 0, i32 0), i64 3 }, %str_slice { i8* getelementptr inbounds ([3 x i8], [3 x i8]* @str4218, i32 0, i32 0), i64 3 }, %str_slice { i8* getelementptr inbounds ([3 x i8], [3 x i8]* @str4219, i32 0, i32 0), i64 3 }, %str_slice { i8* getelementptr inbounds ([3 x i8], [3 x i8]* @str4220, i32 0, i32 0), i64 3 }, %str_slice { i8* getelementptr inbounds ([1 x i8], [1 x i8]* @str4221, i32 0, i32 0), i64 1 }], align 8

; Function Attrs: argmemonly nounwind
declare void @llvm.lifetime.start(i64, i8* nocapture) #0

; Function Attrs: argmemonly nounwind
declare void @llvm.memset.p0i8.i64(i8* nocapture, i8, i64, i32, i1) #0

; Function Attrs: nounwind readnone
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: argmemonly nounwind
declare void @llvm.lifetime.end(i64, i8* nocapture) #0

; Function Attrs: noreturn uwtable
define internal void @_ZN11rdtscp_test4main17h5d687830d7543f82E() unnamed_addr #2 !dbg !198 {
entry-block:
  %a = alloca i64, align 8
  %b = alloca i64, align 8
  %c = alloca i64, align 8
  %d = alloca i32, align 4
  %e1 = alloca i64, align 8
  %f = alloca i32, align 4
  %g = alloca i64, align 8
  %h = alloca i64, align 8
  %i = alloca i64, align 8
  %0 = alloca %"2.std::fmt::Arguments", align 8
  %1 = alloca [9 x %"2.std::fmt::ArgumentV1"], align 8
  %2 = bitcast i64* %a to i8*
  call void @llvm.lifetime.start(i64 8, i8* %2)
  tail call void @llvm.dbg.value(metadata i64 0, i64 0, metadata !200, metadata !260), !dbg !261
  store i64 0, i64* %a, align 8, !dbg !261
  %3 = bitcast i64* %b to i8*
  call void @llvm.lifetime.start(i64 8, i8* %3)
  tail call void @llvm.dbg.value(metadata i64 0, i64 0, metadata !202, metadata !260), !dbg !262
  store i64 0, i64* %b, align 8, !dbg !262
  %4 = bitcast i64* %c to i8*
  call void @llvm.lifetime.start(i64 8, i8* %4)
  tail call void @llvm.dbg.value(metadata i64 0, i64 0, metadata !203, metadata !260), !dbg !263
  store i64 0, i64* %c, align 8, !dbg !263
  %5 = bitcast i32* %d to i8*
  call void @llvm.lifetime.start(i64 4, i8* %5)
  tail call void @llvm.dbg.value(metadata i32 0, i64 0, metadata !204, metadata !260), !dbg !264
  store i32 0, i32* %d, align 4, !dbg !264
  tail call void @llvm.dbg.value(metadata i32 0, i64 0, metadata !205, metadata !260), !dbg !265
  %6 = bitcast i64* %e1 to i8*
  call void @llvm.lifetime.start(i64 8, i8* %6)
  tail call void @llvm.dbg.value(metadata i64 0, i64 0, metadata !206, metadata !260), !dbg !266
  store i64 0, i64* %e1, align 8, !dbg !266
  %7 = bitcast i32* %f to i8*
  call void @llvm.lifetime.start(i64 4, i8* %7)
  tail call void @llvm.dbg.value(metadata i32 0, i64 0, metadata !208, metadata !260), !dbg !267
  store i32 0, i32* %f, align 4, !dbg !267
  %8 = bitcast i64* %g to i8*
  call void @llvm.lifetime.start(i64 8, i8* %8)
  tail call void @llvm.dbg.value(metadata i64 0, i64 0, metadata !209, metadata !260), !dbg !268
  store i64 0, i64* %g, align 8, !dbg !268
  %9 = bitcast i64* %h to i8*
  call void @llvm.lifetime.start(i64 8, i8* %9)
  tail call void @llvm.dbg.value(metadata i64 0, i64 0, metadata !210, metadata !260), !dbg !269
  store i64 0, i64* %h, align 8, !dbg !269
  %10 = bitcast i64* %i to i8*
  call void @llvm.lifetime.start(i64 8, i8* %10)
  tail call void @llvm.dbg.value(metadata i64 0, i64 0, metadata !211, metadata !260), !dbg !270
  store i64 0, i64* %i, align 8, !dbg !270
  %11 = bitcast %"2.std::fmt::Arguments"* %0 to i8*
  %12 = bitcast [9 x %"2.std::fmt::ArgumentV1"]* %1 to i8*
  %13 = getelementptr inbounds [9 x %"2.std::fmt::ArgumentV1"], [9 x %"2.std::fmt::ArgumentV1"]* %1, i64 0, i64 0, !dbg !271
  %14 = getelementptr inbounds [9 x %"2.std::fmt::ArgumentV1"], [9 x %"2.std::fmt::ArgumentV1"]* %1, i64 0, i64 0, i32 1, !dbg !272
  %15 = bitcast [9 x %"2.std::fmt::ArgumentV1"]* %1 to i64**, !dbg !273
  %16 = getelementptr inbounds [9 x %"2.std::fmt::ArgumentV1"], [9 x %"2.std::fmt::ArgumentV1"]* %1, i64 0, i64 1, !dbg !272
  %17 = getelementptr inbounds [9 x %"2.std::fmt::ArgumentV1"], [9 x %"2.std::fmt::ArgumentV1"]* %1, i64 0, i64 1, i32 1, !dbg !272
  %18 = bitcast %"2.std::fmt::ArgumentV1"* %16 to i64**, !dbg !277
  %19 = getelementptr inbounds [9 x %"2.std::fmt::ArgumentV1"], [9 x %"2.std::fmt::ArgumentV1"]* %1, i64 0, i64 2, !dbg !272
  %20 = getelementptr inbounds [9 x %"2.std::fmt::ArgumentV1"], [9 x %"2.std::fmt::ArgumentV1"]* %1, i64 0, i64 2, i32 1, !dbg !272
  %21 = bitcast %"2.std::fmt::ArgumentV1"* %19 to i64**, !dbg !279
  %22 = getelementptr inbounds [9 x %"2.std::fmt::ArgumentV1"], [9 x %"2.std::fmt::ArgumentV1"]* %1, i64 0, i64 3, !dbg !272
  %23 = getelementptr inbounds [9 x %"2.std::fmt::ArgumentV1"], [9 x %"2.std::fmt::ArgumentV1"]* %1, i64 0, i64 3, i32 1, !dbg !272
  %24 = bitcast %"2.std::fmt::ArgumentV1"* %22 to i32**, !dbg !281
  %25 = getelementptr inbounds [9 x %"2.std::fmt::ArgumentV1"], [9 x %"2.std::fmt::ArgumentV1"]* %1, i64 0, i64 4, !dbg !272
  %26 = getelementptr inbounds [9 x %"2.std::fmt::ArgumentV1"], [9 x %"2.std::fmt::ArgumentV1"]* %1, i64 0, i64 4, i32 1, !dbg !272
  %27 = bitcast %"2.std::fmt::ArgumentV1"* %25 to i64**, !dbg !285
  %28 = getelementptr inbounds [9 x %"2.std::fmt::ArgumentV1"], [9 x %"2.std::fmt::ArgumentV1"]* %1, i64 0, i64 5, !dbg !272
  %29 = getelementptr inbounds [9 x %"2.std::fmt::ArgumentV1"], [9 x %"2.std::fmt::ArgumentV1"]* %1, i64 0, i64 5, i32 1, !dbg !272
  %30 = bitcast %"2.std::fmt::ArgumentV1"* %28 to i32**, !dbg !287
  %31 = getelementptr inbounds [9 x %"2.std::fmt::ArgumentV1"], [9 x %"2.std::fmt::ArgumentV1"]* %1, i64 0, i64 6, !dbg !272
  %32 = getelementptr inbounds [9 x %"2.std::fmt::ArgumentV1"], [9 x %"2.std::fmt::ArgumentV1"]* %1, i64 0, i64 6, i32 1, !dbg !272
  %33 = bitcast %"2.std::fmt::ArgumentV1"* %31 to i64**, !dbg !289
  %34 = getelementptr inbounds [9 x %"2.std::fmt::ArgumentV1"], [9 x %"2.std::fmt::ArgumentV1"]* %1, i64 0, i64 7, !dbg !272
  %35 = getelementptr inbounds [9 x %"2.std::fmt::ArgumentV1"], [9 x %"2.std::fmt::ArgumentV1"]* %1, i64 0, i64 7, i32 1, !dbg !272
  %36 = bitcast %"2.std::fmt::ArgumentV1"* %34 to i64**, !dbg !291
  %37 = getelementptr inbounds [9 x %"2.std::fmt::ArgumentV1"], [9 x %"2.std::fmt::ArgumentV1"]* %1, i64 0, i64 8, !dbg !272
  %38 = getelementptr inbounds [9 x %"2.std::fmt::ArgumentV1"], [9 x %"2.std::fmt::ArgumentV1"]* %1, i64 0, i64 8, i32 1, !dbg !272
  %39 = bitcast %"2.std::fmt::ArgumentV1"* %37 to i64**, !dbg !293
  %40 = getelementptr inbounds %"2.std::fmt::Arguments", %"2.std::fmt::Arguments"* %0, i64 0, i32 0, i32 0, !dbg !295
  %41 = getelementptr inbounds %"2.std::fmt::Arguments", %"2.std::fmt::Arguments"* %0, i64 0, i32 0, i32 1, !dbg !295
  %42 = getelementptr inbounds %"2.std::fmt::Arguments", %"2.std::fmt::Arguments"* %0, i64 0, i32 1, !dbg !298
  %43 = bitcast %"2.std::option::Option<&'static [std::fmt::rt::v1::Argument]>"* %42 to i8*, !dbg !299
  %44 = getelementptr inbounds %"2.std::fmt::Arguments", %"2.std::fmt::Arguments"* %0, i64 0, i32 2, i32 0, !dbg !300
  %45 = getelementptr inbounds %"2.std::fmt::Arguments", %"2.std::fmt::Arguments"* %0, i64 0, i32 2, i32 1, !dbg !300
  br label %loop_body, !dbg !301

loop_body:                                        ; preds = %while_exit2, %entry-block
  call void @llvm.dbg.value(metadata i32 488447261, i64 0, metadata !175, metadata !260) #4, !dbg !302
  call void @llvm.dbg.value(metadata i32 488447261, i64 0, metadata !177, metadata !260) #4, !dbg !304
  call void @llvm.dbg.value(metadata i32 488447261, i64 0, metadata !178, metadata !260) #4, !dbg !305
  %46 = call { i32, i32, i32 } asm sideeffect "rdtscp", "={eax},={edx},={ecx},~{dirflag},~{fpsr},~{flags}"() #4, !dbg !306, !srcloc !307
  %47 = extractvalue { i32, i32, i32 } %46, 0, !dbg !306
  call void @llvm.dbg.value(metadata i32 %47, i64 0, metadata !177, metadata !260) #4, !dbg !304
  %48 = extractvalue { i32, i32, i32 } %46, 1, !dbg !306
  call void @llvm.dbg.value(metadata i32 %48, i64 0, metadata !175, metadata !260) #4, !dbg !302
  %49 = zext i32 %48 to i64, !dbg !308
  %50 = shl nuw i64 %49, 32, !dbg !308
  %51 = zext i32 %47 to i64, !dbg !308
  %52 = or i64 %50, %51, !dbg !308
  call void @llvm.dbg.value(metadata i64 %52, i64 0, metadata !179, metadata !260) #4, !dbg !308
  call void @llvm.dbg.value(metadata i64 %52, i64 0, metadata !200, metadata !260), !dbg !261
  store i64 %52, i64* %a, align 8, !dbg !309
  %53 = load i64, i64* %b, align 8, !dbg !310
  call void @llvm.dbg.value(metadata i64 %53, i64 0, metadata !202, metadata !260), !dbg !262
  call void @llvm.dbg.value(metadata i64 %52, i64 0, metadata !200, metadata !260), !dbg !261
  %54 = add i64 %52, 750, !dbg !310
  %55 = icmp ult i64 %53, %54, !dbg !310
  br i1 %55, label %while_body.preheader, label %while_cond3.preheader, !dbg !310

while_body.preheader:                             ; preds = %loop_body
  br label %while_body, !dbg !311

while_cond3.preheader.loopexit:                   ; preds = %while_body
  %.lcssa21 = phi i64 [ %68, %while_body ]
  %.lcssa = phi i64 [ %67, %while_body ]
  br label %while_cond3.preheader, !dbg !314

while_cond3.preheader:                            ; preds = %while_cond3.preheader.loopexit, %loop_body
  %56 = phi i64 [ %52, %loop_body ], [ %.lcssa21, %while_cond3.preheader.loopexit ], !dbg !314
  %57 = phi i64 [ %53, %loop_body ], [ %.lcssa, %while_cond3.preheader.loopexit ], !dbg !314
  %58 = load i64, i64* %c, align 8, !dbg !314
  call void @llvm.dbg.value(metadata i64 %58, i64 0, metadata !203, metadata !260), !dbg !263
  call void @llvm.dbg.value(metadata i64 %57, i64 0, metadata !202, metadata !260), !dbg !262
  call void @llvm.dbg.value(metadata i64 %56, i64 0, metadata !200, metadata !260), !dbg !261
  %factor14 = shl i64 %57, 1
  %59 = sub i64 %factor14, %56, !dbg !314
  %60 = icmp ult i64 %58, %59, !dbg !314
  br i1 %60, label %while_body4.preheader, label %while_exit2, !dbg !314

while_body4.preheader:                            ; preds = %while_cond3.preheader
  br label %while_body4, !dbg !315

while_body:                                       ; preds = %while_body.preheader, %while_body
  call void @llvm.dbg.value(metadata i32 488447261, i64 0, metadata !175, metadata !260) #4, !dbg !311
  call void @llvm.dbg.value(metadata i32 488447261, i64 0, metadata !177, metadata !260) #4, !dbg !318
  call void @llvm.dbg.value(metadata i32 488447261, i64 0, metadata !178, metadata !260) #4, !dbg !319
  %61 = call { i32, i32, i32 } asm sideeffect "rdtscp", "={eax},={edx},={ecx},~{dirflag},~{fpsr},~{flags}"() #4, !dbg !320, !srcloc !307
  %62 = extractvalue { i32, i32, i32 } %61, 0, !dbg !320
  call void @llvm.dbg.value(metadata i32 %62, i64 0, metadata !177, metadata !260) #4, !dbg !318
  %63 = extractvalue { i32, i32, i32 } %61, 1, !dbg !320
  call void @llvm.dbg.value(metadata i32 %63, i64 0, metadata !175, metadata !260) #4, !dbg !311
  %64 = zext i32 %63 to i64, !dbg !321
  %65 = shl nuw i64 %64, 32, !dbg !321
  %66 = zext i32 %62 to i64, !dbg !321
  %67 = or i64 %65, %66, !dbg !321
  call void @llvm.dbg.value(metadata i64 %67, i64 0, metadata !179, metadata !260) #4, !dbg !321
  call void @llvm.dbg.value(metadata i64 %67, i64 0, metadata !202, metadata !260), !dbg !262
  store i64 %67, i64* %b, align 8, !dbg !322
  call void @llvm.dbg.value(metadata i64 %67, i64 0, metadata !202, metadata !260), !dbg !262
  %68 = load i64, i64* %a, align 8, !dbg !310
  call void @llvm.dbg.value(metadata i64 %68, i64 0, metadata !200, metadata !260), !dbg !261
  %69 = add i64 %68, 750, !dbg !310
  %70 = icmp ult i64 %67, %69, !dbg !310
  br i1 %70, label %while_body, label %while_cond3.preheader.loopexit, !dbg !310

while_exit2.loopexit:                             ; preds = %while_body4
  %.lcssa22 = phi i64 [ %97, %while_body4 ]
  br label %while_exit2, !dbg !323

while_exit2:                                      ; preds = %while_exit2.loopexit, %while_cond3.preheader
  %71 = phi i64 [ %57, %while_cond3.preheader ], [ %.lcssa22, %while_exit2.loopexit ], !dbg !324
  %72 = load i32, i32* %d, align 4, !dbg !323
  call void @llvm.dbg.value(metadata i32 %72, i64 0, metadata !204, metadata !260), !dbg !264
  %73 = add i32 %72, 1, !dbg !323
  call void @llvm.dbg.value(metadata i32 %73, i64 0, metadata !204, metadata !260), !dbg !264
  store i32 %73, i32* %d, align 4, !dbg !323
  %74 = load i64, i64* %e1, align 8, !dbg !324
  call void @llvm.dbg.value(metadata i64 %74, i64 0, metadata !206, metadata !260), !dbg !266
  call void @llvm.dbg.value(metadata i64 %71, i64 0, metadata !202, metadata !260), !dbg !262
  %75 = shl i64 %71, 1, !dbg !324
  %76 = add i64 %75, %74, !dbg !324
  call void @llvm.dbg.value(metadata i64 %76, i64 0, metadata !206, metadata !260), !dbg !266
  store i64 %76, i64* %e1, align 8, !dbg !324
  call void @llvm.dbg.value(metadata i32 0, i64 0, metadata !191, metadata !260) #4, !dbg !325
  call void @llvm.dbg.value(metadata i32 0, i64 0, metadata !195, metadata !260) #4, !dbg !327
  call void @llvm.dbg.value(metadata i32 0, i64 0, metadata !196, metadata !260) #4, !dbg !328
  call void @llvm.dbg.value(metadata i32 0, i64 0, metadata !197, metadata !260) #4, !dbg !329
  call void asm sideeffect "movl $$0x2, %eax", "~{eax},~{dirflag},~{fpsr},~{flags}"() #4, !dbg !330, !srcloc !331
  call void asm sideeffect "movl $$0x0, %ecx", "~{ecx},~{dirflag},~{fpsr},~{flags}"() #4, !dbg !332, !srcloc !333
  %77 = call { i32, i32, i32, i32 } asm sideeffect "cpuid", "={rax},={rbx},={rcx},={rdx},~{dirflag},~{fpsr},~{flags}"() #4, !dbg !334, !srcloc !335
  call void @llvm.dbg.value(metadata i32 488447261, i64 0, metadata !175, metadata !260) #4, !dbg !336
  call void @llvm.dbg.value(metadata i32 488447261, i64 0, metadata !177, metadata !260) #4, !dbg !338
  call void @llvm.dbg.value(metadata i32 488447261, i64 0, metadata !178, metadata !260) #4, !dbg !339
  %78 = call { i32, i32, i32 } asm sideeffect "rdtscp", "={eax},={edx},={ecx},~{dirflag},~{fpsr},~{flags}"() #4, !dbg !340, !srcloc !307
  %79 = extractvalue { i32, i32, i32 } %78, 0, !dbg !340
  call void @llvm.dbg.value(metadata i32 %79, i64 0, metadata !177, metadata !260) #4, !dbg !338
  %80 = extractvalue { i32, i32, i32 } %78, 1, !dbg !340
  call void @llvm.dbg.value(metadata i32 %80, i64 0, metadata !175, metadata !260) #4, !dbg !336
  %81 = zext i32 %80 to i64, !dbg !341
  %82 = shl nuw i64 %81, 32, !dbg !341
  %83 = zext i32 %79 to i64, !dbg !341
  %84 = or i64 %82, %83, !dbg !341
  call void @llvm.dbg.value(metadata i64 %84, i64 0, metadata !179, metadata !260) #4, !dbg !341
  call void @llvm.dbg.value(metadata i64 %84, i64 0, metadata !209, metadata !260), !dbg !268
  store i64 %84, i64* %g, align 8, !dbg !342
  call void @llvm.dbg.value(metadata i32 0, i64 0, metadata !191, metadata !260) #4, !dbg !343
  call void @llvm.dbg.value(metadata i32 0, i64 0, metadata !195, metadata !260) #4, !dbg !345
  call void @llvm.dbg.value(metadata i32 0, i64 0, metadata !196, metadata !260) #4, !dbg !346
  call void @llvm.dbg.value(metadata i32 0, i64 0, metadata !197, metadata !260) #4, !dbg !347
  call void asm sideeffect "movl $$0x2, %eax", "~{eax},~{dirflag},~{fpsr},~{flags}"() #4, !dbg !348, !srcloc !331
  call void asm sideeffect "movl $$0x0, %ecx", "~{ecx},~{dirflag},~{fpsr},~{flags}"() #4, !dbg !349, !srcloc !333
  %85 = call { i32, i32, i32, i32 } asm sideeffect "cpuid", "={rax},={rbx},={rcx},={rdx},~{dirflag},~{fpsr},~{flags}"() #4, !dbg !350, !srcloc !335
  %86 = load i64, i64* %g, align 8, !dbg !351
  call void @llvm.dbg.value(metadata i64 %86, i64 0, metadata !209, metadata !260), !dbg !268
  %87 = add i64 %86, 2, !dbg !351
  call void @llvm.dbg.value(metadata i64 %87, i64 0, metadata !210, metadata !260), !dbg !269
  store i64 %87, i64* %h, align 8, !dbg !351
  %88 = load i64, i64* %a, align 8, !dbg !352
  call void @llvm.dbg.value(metadata i64 %88, i64 0, metadata !200, metadata !260), !dbg !261
  call void @llvm.dbg.value(metadata i64 %86, i64 0, metadata !209, metadata !260), !dbg !268
  %89 = add i64 %88, %86, !dbg !352
  call void @llvm.dbg.value(metadata i64 %89, i64 0, metadata !211, metadata !260), !dbg !270
  store i64 %89, i64* %i, align 8, !dbg !352
  call void @llvm.lifetime.start(i64 48, i8* %11)
  call void @llvm.lifetime.start(i64 144, i8* %12)
  call void @llvm.dbg.value(metadata i32* %f, i64 0, metadata !212, metadata !260), !dbg !353
  call void @llvm.dbg.value(metadata i64* %b, i64 0, metadata !217, metadata !260), !dbg !353
  call void @llvm.dbg.value(metadata i64* %i, i64 0, metadata !219, metadata !260), !dbg !353
  call void @llvm.dbg.value(metadata i64* %e1, i64 0, metadata !220, metadata !260), !dbg !353
  call void @llvm.dbg.value(metadata i64* %h, i64 0, metadata !221, metadata !260), !dbg !353
  call void @llvm.dbg.value(metadata i64* %g, i64 0, metadata !222, metadata !260), !dbg !353
  call void @llvm.dbg.value(metadata i32* %d, i64 0, metadata !223, metadata !260), !dbg !353
  call void @llvm.dbg.value(metadata i64* %c, i64 0, metadata !224, metadata !260), !dbg !353
  call void @llvm.dbg.value(metadata i64* %a, i64 0, metadata !225, metadata !260), !dbg !353
  call void @llvm.dbg.value(metadata i64* %a, i64 0, metadata !200, metadata !354), !dbg !261
  tail call void @llvm.dbg.value(metadata i8 (i64*, %"2.std::fmt::Formatter"*)* @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$u64$GT$3fmt17hb7b8526a4b4a5b3eE", i64 0, metadata !244, metadata !260), !dbg !355
  store i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)* bitcast (i8 (i64*, %"2.std::fmt::Formatter"*)* @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$u64$GT$3fmt17hb7b8526a4b4a5b3eE" to i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)*), i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)** %14, align 8, !dbg !356, !alias.scope !357, !noalias !360
  store i64* %a, i64** %15, align 8, !dbg !273, !alias.scope !357, !noalias !360
  call void @llvm.dbg.value(metadata i64* %b, i64 0, metadata !202, metadata !354), !dbg !262
  tail call void @llvm.dbg.value(metadata i8 (i64*, %"2.std::fmt::Formatter"*)* @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$u64$GT$3fmt17hb7b8526a4b4a5b3eE", i64 0, metadata !244, metadata !260), !dbg !362
  store i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)* bitcast (i8 (i64*, %"2.std::fmt::Formatter"*)* @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$u64$GT$3fmt17hb7b8526a4b4a5b3eE" to i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)*), i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)** %17, align 8, !dbg !363, !alias.scope !364, !noalias !367
  store i64* %b, i64** %18, align 8, !dbg !277, !alias.scope !364, !noalias !367
  call void @llvm.dbg.value(metadata i64* %c, i64 0, metadata !203, metadata !354), !dbg !263
  tail call void @llvm.dbg.value(metadata i8 (i64*, %"2.std::fmt::Formatter"*)* @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$u64$GT$3fmt17hb7b8526a4b4a5b3eE", i64 0, metadata !244, metadata !260), !dbg !369
  store i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)* bitcast (i8 (i64*, %"2.std::fmt::Formatter"*)* @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$u64$GT$3fmt17hb7b8526a4b4a5b3eE" to i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)*), i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)** %20, align 8, !dbg !370, !alias.scope !371, !noalias !374
  store i64* %c, i64** %21, align 8, !dbg !279, !alias.scope !371, !noalias !374
  call void @llvm.dbg.value(metadata i32* %d, i64 0, metadata !204, metadata !354), !dbg !264
  tail call void @llvm.dbg.value(metadata i8 (i32*, %"2.std::fmt::Formatter"*)* @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$i32$GT$3fmt17h61d5cba22cbe298eE", i64 0, metadata !255, metadata !260), !dbg !376
  store i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)* bitcast (i8 (i32*, %"2.std::fmt::Formatter"*)* @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$i32$GT$3fmt17h61d5cba22cbe298eE" to i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)*), i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)** %23, align 8, !dbg !377, !alias.scope !378, !noalias !381
  store i32* %d, i32** %24, align 8, !dbg !281, !alias.scope !378, !noalias !381
  call void @llvm.dbg.value(metadata i64* %e1, i64 0, metadata !206, metadata !354), !dbg !266
  tail call void @llvm.dbg.value(metadata i8 (i64*, %"2.std::fmt::Formatter"*)* @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$u64$GT$3fmt17hb7b8526a4b4a5b3eE", i64 0, metadata !244, metadata !260), !dbg !383
  store i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)* bitcast (i8 (i64*, %"2.std::fmt::Formatter"*)* @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$u64$GT$3fmt17hb7b8526a4b4a5b3eE" to i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)*), i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)** %26, align 8, !dbg !384, !alias.scope !385, !noalias !388
  store i64* %e1, i64** %27, align 8, !dbg !285, !alias.scope !385, !noalias !388
  call void @llvm.dbg.value(metadata i32* %f, i64 0, metadata !208, metadata !354), !dbg !267
  tail call void @llvm.dbg.value(metadata i8 (i32*, %"2.std::fmt::Formatter"*)* @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$i32$GT$3fmt17h61d5cba22cbe298eE", i64 0, metadata !255, metadata !260), !dbg !390
  store i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)* bitcast (i8 (i32*, %"2.std::fmt::Formatter"*)* @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$i32$GT$3fmt17h61d5cba22cbe298eE" to i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)*), i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)** %29, align 8, !dbg !391, !alias.scope !392, !noalias !395
  store i32* %f, i32** %30, align 8, !dbg !287, !alias.scope !392, !noalias !395
  call void @llvm.dbg.value(metadata i64* %g, i64 0, metadata !209, metadata !354), !dbg !268
  tail call void @llvm.dbg.value(metadata i8 (i64*, %"2.std::fmt::Formatter"*)* @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$u64$GT$3fmt17hb7b8526a4b4a5b3eE", i64 0, metadata !244, metadata !260), !dbg !397
  store i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)* bitcast (i8 (i64*, %"2.std::fmt::Formatter"*)* @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$u64$GT$3fmt17hb7b8526a4b4a5b3eE" to i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)*), i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)** %32, align 8, !dbg !398, !alias.scope !399, !noalias !402
  store i64* %g, i64** %33, align 8, !dbg !289, !alias.scope !399, !noalias !402
  call void @llvm.dbg.value(metadata i64* %h, i64 0, metadata !210, metadata !354), !dbg !269
  tail call void @llvm.dbg.value(metadata i8 (i64*, %"2.std::fmt::Formatter"*)* @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$u64$GT$3fmt17hb7b8526a4b4a5b3eE", i64 0, metadata !244, metadata !260), !dbg !404
  store i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)* bitcast (i8 (i64*, %"2.std::fmt::Formatter"*)* @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$u64$GT$3fmt17hb7b8526a4b4a5b3eE" to i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)*), i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)** %35, align 8, !dbg !405, !alias.scope !406, !noalias !409
  store i64* %h, i64** %36, align 8, !dbg !291, !alias.scope !406, !noalias !409
  call void @llvm.dbg.value(metadata i64* %i, i64 0, metadata !211, metadata !354), !dbg !270
  tail call void @llvm.dbg.value(metadata i8 (i64*, %"2.std::fmt::Formatter"*)* @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$u64$GT$3fmt17hb7b8526a4b4a5b3eE", i64 0, metadata !244, metadata !260), !dbg !411
  store i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)* bitcast (i8 (i64*, %"2.std::fmt::Formatter"*)* @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$u64$GT$3fmt17hb7b8526a4b4a5b3eE" to i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)*), i8 (%"2.core::fmt::Void"*, %"2.std::fmt::Formatter"*)** %38, align 8, !dbg !412, !alias.scope !413, !noalias !416
  store i64* %i, i64** %39, align 8, !dbg !293, !alias.scope !413, !noalias !416
  call void @llvm.dbg.value(metadata !122, i64 0, metadata !232, metadata !418) #4, !dbg !419
  call void @llvm.dbg.value(metadata i64 10, i64 0, metadata !232, metadata !420) #4, !dbg !419
  call void @llvm.dbg.declare(metadata { %str_slice*, i64 }* undef, metadata !232, metadata !260) #4, !dbg !419
  call void @llvm.dbg.value(metadata i64 9, i64 0, metadata !233, metadata !420) #4, !dbg !421
  call void @llvm.dbg.declare(metadata { %"2.std::fmt::ArgumentV1"*, i64 }* undef, metadata !233, metadata !260) #4, !dbg !421
  store %str_slice* getelementptr inbounds ([10 x %str_slice], [10 x %str_slice]* @ref4222, i64 0, i64 0), %str_slice** %40, align 8, !dbg !295, !alias.scope !422, !noalias !425
  store i64 10, i64* %41, align 8, !dbg !295, !alias.scope !422, !noalias !425
  call void @llvm.memset.p0i8.i64(i8* %43, i8 0, i64 16, i32 8, i1 false) #4, !dbg !299, !alias.scope !422, !noalias !425
  store %"2.std::fmt::ArgumentV1"* %13, %"2.std::fmt::ArgumentV1"** %44, align 8, !dbg !300, !alias.scope !422, !noalias !425
  store i64 9, i64* %45, align 8, !dbg !300, !alias.scope !422, !noalias !425
  call void @_ZN3std2io5stdio6_print17h248ab2ee448482f3E(%"2.std::fmt::Arguments"* noalias nocapture nonnull dereferenceable(48) %0), !dbg !298
  call void @llvm.lifetime.end(i64 48, i8* %11)
  call void @llvm.lifetime.end(i64 144, i8* %12)
  br label %loop_body

while_body4:                                      ; preds = %while_body4.preheader, %while_body4
  call void @llvm.dbg.value(metadata i32 488447261, i64 0, metadata !175, metadata !260) #4, !dbg !315
  call void @llvm.dbg.value(metadata i32 488447261, i64 0, metadata !177, metadata !260) #4, !dbg !427
  call void @llvm.dbg.value(metadata i32 488447261, i64 0, metadata !178, metadata !260) #4, !dbg !428
  %90 = call { i32, i32, i32 } asm sideeffect "rdtscp", "={eax},={edx},={ecx},~{dirflag},~{fpsr},~{flags}"() #4, !dbg !429, !srcloc !307
  %91 = extractvalue { i32, i32, i32 } %90, 0, !dbg !429
  call void @llvm.dbg.value(metadata i32 %91, i64 0, metadata !177, metadata !260) #4, !dbg !427
  %92 = extractvalue { i32, i32, i32 } %90, 1, !dbg !429
  call void @llvm.dbg.value(metadata i32 %92, i64 0, metadata !175, metadata !260) #4, !dbg !315
  %93 = zext i32 %92 to i64, !dbg !430
  %94 = shl nuw i64 %93, 32, !dbg !430
  %95 = zext i32 %91 to i64, !dbg !430
  %96 = or i64 %94, %95, !dbg !430
  call void @llvm.dbg.value(metadata i64 %96, i64 0, metadata !179, metadata !260) #4, !dbg !430
  call void @llvm.dbg.value(metadata i64 %96, i64 0, metadata !203, metadata !260), !dbg !263
  store i64 %96, i64* %c, align 8, !dbg !431
  call void @llvm.dbg.value(metadata i64 %96, i64 0, metadata !203, metadata !260), !dbg !263
  %97 = load i64, i64* %b, align 8, !dbg !314
  call void @llvm.dbg.value(metadata i64 %97, i64 0, metadata !202, metadata !260), !dbg !262
  %98 = load i64, i64* %a, align 8, !dbg !314
  call void @llvm.dbg.value(metadata i64 %98, i64 0, metadata !200, metadata !260), !dbg !261
  %factor = shl i64 %97, 1
  %99 = sub i64 %factor, %98, !dbg !314
  %100 = icmp ult i64 %96, %99, !dbg !314
  br i1 %100, label %while_body4, label %while_exit2.loopexit, !dbg !314
}

declare void @_ZN3std2io5stdio6_print17h248ab2ee448482f3E(%"2.std::fmt::Arguments"* noalias nocapture dereferenceable(48)) unnamed_addr #3

declare i8 @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$u64$GT$3fmt17hb7b8526a4b4a5b3eE"(i64* noalias readonly dereferenceable(8), %"2.std::fmt::Formatter"* dereferenceable(96)) unnamed_addr #3

declare i8 @"_ZN4core3fmt3num46_$LT$impl$u20$fmt..Display$u20$for$u20$i32$GT$3fmt17h61d5cba22cbe298eE"(i32* noalias readonly dereferenceable(4), %"2.std::fmt::Formatter"* dereferenceable(96)) unnamed_addr #3

define i64 @main(i64, i8**) unnamed_addr {
top:
  %2 = load volatile i8, i8* getelementptr inbounds ([34 x i8], [34 x i8]* @__rustc_debug_gdb_scripts_section__, i64 0, i64 0), align 1
  %3 = tail call i64 @_ZN3std2rt10lang_start17h801b666f82634252E(i8* bitcast (void ()* @_ZN11rdtscp_test4main17h5d687830d7543f82E to i8*), i64 %0, i8** %1)
  ret i64 %3
}

declare i64 @_ZN3std2rt10lang_start17h801b666f82634252E(i8*, i64, i8**) unnamed_addr #3

; Function Attrs: nounwind readnone
declare void @llvm.dbg.value(metadata, i64, metadata, metadata) #1

attributes #0 = { argmemonly nounwind }
attributes #1 = { nounwind readnone }
attributes #2 = { noreturn uwtable "no-frame-pointer-elim"="true" }
attributes #3 = { "no-frame-pointer-elim"="true" }
attributes #4 = { nounwind }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!259}

!0 = distinct !DICompileUnit(language: DW_LANG_Rust, file: !1, producer: "rustc version 1.10.0-dev (700279f4b 2016-05-11)", isOptimized: true, runtimeVersion: 0, emissionKind: 1, enums: !2, retainedTypes: !35, subprograms: !168, globals: !256)
!1 = !DIFile(filename: "./src/main.rs", directory: "/home/apanda/e2d2/debugging/rdtscp-test")
!2 = !{!3, !12, !19, !25, !30}
!3 = !DICompositeType(tag: DW_TAG_enumeration_type, name: "Position", scope: !4, baseType: !8, size: 64, align: 64, elements: !9)
!4 = !DINamespace(name: "v1", scope: !5)
!5 = !DINamespace(name: "rt", scope: !6)
!6 = !DINamespace(name: "fmt", scope: !7)
!7 = !DINamespace(name: "core", scope: null)
!8 = !DIBasicType(name: "u64", size: 64, align: 64, encoding: DW_ATE_unsigned)
!9 = !{!10, !11}
!10 = !DIEnumerator(name: "Next", value: 0)
!11 = !DIEnumerator(name: "At", value: 1)
!12 = !DICompositeType(tag: DW_TAG_enumeration_type, name: "Alignment", scope: !4, baseType: !13, size: 8, align: 8, elements: !14)
!13 = !DIBasicType(name: "u8", size: 8, align: 8, encoding: DW_ATE_unsigned)
!14 = !{!15, !16, !17, !18}
!15 = !DIEnumerator(name: "Left", value: 0)
!16 = !DIEnumerator(name: "Right", value: 1)
!17 = !DIEnumerator(name: "Center", value: 2)
!18 = !DIEnumerator(name: "Unknown", value: 3)
!19 = !DICompositeType(tag: DW_TAG_enumeration_type, name: "Count", scope: !4, baseType: !8, size: 64, align: 64, elements: !20)
!20 = !{!21, !22, !23, !24}
!21 = !DIEnumerator(name: "Is", value: 0)
!22 = !DIEnumerator(name: "Param", value: 1)
!23 = !DIEnumerator(name: "NextParam", value: 2)
!24 = !DIEnumerator(name: "Implied", value: 3)
!25 = !DICompositeType(tag: DW_TAG_enumeration_type, name: "Result", scope: !26, baseType: !13, size: 8, align: 8, elements: !27)
!26 = !DINamespace(name: "result", scope: !7)
!27 = !{!28, !29}
!28 = !DIEnumerator(name: "Ok", value: 0)
!29 = !DIEnumerator(name: "Err", value: 1)
!30 = !DICompositeType(tag: DW_TAG_enumeration_type, name: "Option", scope: !31, baseType: !8, size: 64, align: 64, elements: !32)
!31 = !DINamespace(name: "option", scope: !7)
!32 = !{!33, !34}
!33 = !DIEnumerator(name: "None", value: 0)
!34 = !DIEnumerator(name: "Some", value: 1)
!35 = !{!36, !41, !47, !52, !56, !59, !63, !67, !70, !74, !83, !89, !93, !97, !100, !103, !108, !121, !116, !123, !128, !132, !133, !143, !147, !150, !154, !155, !162, !164}
!36 = !DICompositeType(tag: DW_TAG_structure_type, name: "Arguments", scope: !6, size: 384, align: 64, elements: !37, identifier: "{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/93b8}")
!37 = !{!38, !39, !40}
!38 = !DIDerivedType(tag: DW_TAG_member, name: "pieces", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/93b8}", baseType: !"{&{[]{&{str}}}}", size: 128, align: 64)
!39 = !DIDerivedType(tag: DW_TAG_member, name: "fmt", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/93b8}", baseType: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/7eef<{&{[]{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/92f9}}},>}", size: 128, align: 64, offset: 128)
!40 = !DIDerivedType(tag: DW_TAG_member, name: "args", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/93b8}", baseType: !"{&{[]{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/938a}}}", size: 128, align: 64, offset: 256)
!41 = !DICompositeType(tag: DW_TAG_structure_type, name: "&str", size: 128, align: 64, elements: !42, identifier: "{&{str}}")
!42 = !{!43, !45}
!43 = !DIDerivedType(tag: DW_TAG_member, name: "data_ptr", scope: !"{&{str}}", baseType: !44, size: 64, align: 64)
!44 = !DIDerivedType(tag: DW_TAG_pointer_type, name: "*const u8", baseType: !13, size: 64, align: 64)
!45 = !DIDerivedType(tag: DW_TAG_member, name: "length", scope: !"{&{str}}", baseType: !46, size: 64, align: 64, offset: 64)
!46 = !DIBasicType(name: "usize", size: 64, align: 64, encoding: DW_ATE_unsigned)
!47 = !DICompositeType(tag: DW_TAG_structure_type, name: "&[&str]", size: 128, align: 64, elements: !48, identifier: "{&{[]{&{str}}}}")
!48 = !{!49, !51}
!49 = !DIDerivedType(tag: DW_TAG_member, name: "data_ptr", scope: !"{&{[]{&{str}}}}", baseType: !50, size: 64, align: 64)
!50 = !DIDerivedType(tag: DW_TAG_pointer_type, name: "*const &str", baseType: !"{&{str}}", size: 64, align: 64)
!51 = !DIDerivedType(tag: DW_TAG_member, name: "length", scope: !"{&{[]{&{str}}}}", baseType: !46, size: 64, align: 64, offset: 64)
!52 = !DICompositeType(tag: DW_TAG_union_type, name: "Option<&[core::fmt::rt::v1::Argument]>", scope: !31, file: !53, size: 128, align: 64, elements: !54, identifier: "{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/7eef<{&{[]{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/92f9}}},>}")
!53 = !DIFile(filename: "<unknown>", directory: "")
!54 = !{!55}
!55 = !DIDerivedType(tag: DW_TAG_member, name: "RUST$ENCODED$ENUM$0$0$None", scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/7eef<{&{[]{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/92f9}}},>}", baseType: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/7eef<{&{[]{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/92f9}}},>}::Some", size: 128, align: 64)
!56 = !DICompositeType(tag: DW_TAG_structure_type, name: "Some", scope: !31, size: 128, align: 64, elements: !57, identifier: "{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/7eef<{&{[]{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/92f9}}},>}::Some")
!57 = !{!58}
!58 = !DIDerivedType(tag: DW_TAG_member, name: "__0", scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/7eef<{&{[]{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/92f9}}},>}::Some", baseType: !"{&{[]{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/92f9}}}", size: 128, align: 64)
!59 = !DICompositeType(tag: DW_TAG_structure_type, name: "Argument", scope: !4, size: 512, align: 64, elements: !60, identifier: "{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/92f9}")
!60 = !{!61, !62}
!61 = !DIDerivedType(tag: DW_TAG_member, name: "position", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/92f9}", baseType: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/932d}", size: 128, align: 64)
!62 = !DIDerivedType(tag: DW_TAG_member, name: "format", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/92f9}", baseType: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9302}", size: 384, align: 64, offset: 128)
!63 = !DICompositeType(tag: DW_TAG_union_type, name: "Position", scope: !4, file: !53, size: 128, align: 64, elements: !64, identifier: "{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/932d}")
!64 = !{!65, !66}
!65 = !DIDerivedType(tag: DW_TAG_member, scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/932d}", baseType: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/932d}::Next", size: 64, align: 64)
!66 = !DIDerivedType(tag: DW_TAG_member, scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/932d}", baseType: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/932d}::At", size: 128, align: 64)
!67 = !DICompositeType(tag: DW_TAG_structure_type, name: "Next", scope: !4, size: 64, align: 64, elements: !68, identifier: "{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/932d}::Next")
!68 = !{!69}
!69 = !DIDerivedType(tag: DW_TAG_member, name: "RUST$ENUM$DISR", scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/932d}::Next", baseType: !3, size: 64, align: 64)
!70 = !DICompositeType(tag: DW_TAG_structure_type, name: "At", scope: !4, size: 128, align: 64, elements: !71, identifier: "{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/932d}::At")
!71 = !{!72, !73}
!72 = !DIDerivedType(tag: DW_TAG_member, name: "RUST$ENUM$DISR", scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/932d}::At", baseType: !3, size: 64, align: 64)
!73 = !DIDerivedType(tag: DW_TAG_member, name: "__0", scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/932d}::At", baseType: !46, size: 64, align: 64, offset: 64)
!74 = !DICompositeType(tag: DW_TAG_structure_type, name: "FormatSpec", scope: !4, size: 384, align: 64, elements: !75, identifier: "{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9302}")
!75 = !{!76, !78, !79, !81, !82}
!76 = !DIDerivedType(tag: DW_TAG_member, name: "fill", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9302}", baseType: !77, size: 32, align: 32)
!77 = !DIBasicType(name: "char", size: 32, align: 32, encoding: DW_ATE_unsigned_char)
!78 = !DIDerivedType(tag: DW_TAG_member, name: "align", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9302}", baseType: !12, size: 8, align: 8, offset: 32)
!79 = !DIDerivedType(tag: DW_TAG_member, name: "flags", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9302}", baseType: !80, size: 32, align: 32, offset: 64)
!80 = !DIBasicType(name: "u32", size: 32, align: 32, encoding: DW_ATE_unsigned)
!81 = !DIDerivedType(tag: DW_TAG_member, name: "precision", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9302}", baseType: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}", size: 128, align: 64, offset: 128)
!82 = !DIDerivedType(tag: DW_TAG_member, name: "width", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9302}", baseType: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}", size: 128, align: 64, offset: 256)
!83 = !DICompositeType(tag: DW_TAG_union_type, name: "Count", scope: !4, file: !53, size: 128, align: 64, elements: !84, identifier: "{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}")
!84 = !{!85, !86, !87, !88}
!85 = !DIDerivedType(tag: DW_TAG_member, scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}", baseType: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}::Is", size: 128, align: 64)
!86 = !DIDerivedType(tag: DW_TAG_member, scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}", baseType: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}::Param", size: 128, align: 64)
!87 = !DIDerivedType(tag: DW_TAG_member, scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}", baseType: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}::NextParam", size: 64, align: 64)
!88 = !DIDerivedType(tag: DW_TAG_member, scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}", baseType: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}::Implied", size: 64, align: 64)
!89 = !DICompositeType(tag: DW_TAG_structure_type, name: "Is", scope: !4, size: 128, align: 64, elements: !90, identifier: "{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}::Is")
!90 = !{!91, !92}
!91 = !DIDerivedType(tag: DW_TAG_member, name: "RUST$ENUM$DISR", scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}::Is", baseType: !19, size: 64, align: 64)
!92 = !DIDerivedType(tag: DW_TAG_member, name: "__0", scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}::Is", baseType: !46, size: 64, align: 64, offset: 64)
!93 = !DICompositeType(tag: DW_TAG_structure_type, name: "Param", scope: !4, size: 128, align: 64, elements: !94, identifier: "{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}::Param")
!94 = !{!95, !96}
!95 = !DIDerivedType(tag: DW_TAG_member, name: "RUST$ENUM$DISR", scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}::Param", baseType: !19, size: 64, align: 64)
!96 = !DIDerivedType(tag: DW_TAG_member, name: "__0", scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}::Param", baseType: !46, size: 64, align: 64, offset: 64)
!97 = !DICompositeType(tag: DW_TAG_structure_type, name: "NextParam", scope: !4, size: 64, align: 64, elements: !98, identifier: "{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}::NextParam")
!98 = !{!99}
!99 = !DIDerivedType(tag: DW_TAG_member, name: "RUST$ENUM$DISR", scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}::NextParam", baseType: !19, size: 64, align: 64)
!100 = !DICompositeType(tag: DW_TAG_structure_type, name: "Implied", scope: !4, size: 64, align: 64, elements: !101, identifier: "{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}::Implied")
!101 = !{!102}
!102 = !DIDerivedType(tag: DW_TAG_member, name: "RUST$ENUM$DISR", scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9320}::Implied", baseType: !19, size: 64, align: 64)
!103 = !DICompositeType(tag: DW_TAG_structure_type, name: "&[core::fmt::rt::v1::Argument]", size: 128, align: 64, elements: !104, identifier: "{&{[]{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/92f9}}}")
!104 = !{!105, !107}
!105 = !DIDerivedType(tag: DW_TAG_member, name: "data_ptr", scope: !"{&{[]{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/92f9}}}", baseType: !106, size: 64, align: 64)
!106 = !DIDerivedType(tag: DW_TAG_pointer_type, name: "*const core::fmt::rt::v1::Argument", baseType: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/92f9}", size: 64, align: 64)
!107 = !DIDerivedType(tag: DW_TAG_member, name: "length", scope: !"{&{[]{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/92f9}}}", baseType: !46, size: 64, align: 64, offset: 64)
!108 = !DICompositeType(tag: DW_TAG_structure_type, name: "ArgumentV1", scope: !6, size: 128, align: 64, elements: !109, identifier: "{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/938a}")
!109 = !{!110, !112}
!110 = !DIDerivedType(tag: DW_TAG_member, name: "value", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/938a}", baseType: !111, size: 64, align: 64)
!111 = !DIDerivedType(tag: DW_TAG_pointer_type, name: "&core::fmt::Void", baseType: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9389}", size: 64, align: 64)
!112 = !DIDerivedType(tag: DW_TAG_member, name: "formatter", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/938a}", baseType: !113, size: 64, align: 64, offset: 64)
!113 = !DIDerivedType(tag: DW_TAG_pointer_type, name: "fn(&core::fmt::Void, &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error>", baseType: !114, size: 64, align: 64)
!114 = !DISubroutineType(types: !115)
!115 = !{!116, !111, !120}
!116 = !DICompositeType(tag: DW_TAG_union_type, name: "Result<(), core::fmt::Error>", scope: !26, file: !53, size: 8, align: 8, elements: !117, identifier: "{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/808c<{()},{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9337},>}")
!117 = !{!118, !119}
!118 = !DIDerivedType(tag: DW_TAG_member, scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/808c<{()},{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9337},>}", baseType: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/808c<{()},{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9337},>}::Ok", size: 8, align: 8)
!119 = !DIDerivedType(tag: DW_TAG_member, scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/808c<{()},{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9337},>}", baseType: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/808c<{()},{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9337},>}::Err", size: 8, align: 8)
!120 = !DIDerivedType(tag: DW_TAG_pointer_type, name: "&mut core::fmt::Formatter", baseType: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/937f}", size: 64, align: 64)
!121 = !DICompositeType(tag: DW_TAG_union_type, name: "Void", scope: !6, file: !53, align: 8, elements: !122, identifier: "{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9389}")
!122 = !{}
!123 = !DICompositeType(tag: DW_TAG_structure_type, name: "Ok", scope: !26, size: 8, align: 8, elements: !124, identifier: "{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/808c<{()},{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9337},>}::Ok")
!124 = !{!125, !126}
!125 = !DIDerivedType(tag: DW_TAG_member, name: "RUST$ENUM$DISR", scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/808c<{()},{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9337},>}::Ok", baseType: !25, size: 8, align: 8)
!126 = !DIDerivedType(tag: DW_TAG_member, name: "__0", scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/808c<{()},{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9337},>}::Ok", baseType: !127, align: 8, offset: 8)
!127 = !DIBasicType(name: "()", align: 8, encoding: DW_ATE_unsigned)
!128 = !DICompositeType(tag: DW_TAG_structure_type, name: "Err", scope: !26, size: 8, align: 8, elements: !129, identifier: "{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/808c<{()},{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9337},>}::Err")
!129 = !{!130, !131}
!130 = !DIDerivedType(tag: DW_TAG_member, name: "RUST$ENUM$DISR", scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/808c<{()},{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9337},>}::Err", baseType: !25, size: 8, align: 8)
!131 = !DIDerivedType(tag: DW_TAG_member, name: "__0", scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/808c<{()},{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9337},>}::Err", baseType: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9337}", align: 8, offset: 8)
!132 = !DICompositeType(tag: DW_TAG_structure_type, name: "Error", scope: !6, align: 8, elements: !122, identifier: "{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9337}")
!133 = !DICompositeType(tag: DW_TAG_structure_type, name: "Formatter", scope: !6, size: 768, align: 64, elements: !134, identifier: "{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/937f}")
!134 = !{!135, !136, !137, !138, !139, !140, !141, !142}
!135 = !DIDerivedType(tag: DW_TAG_member, name: "flags", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/937f}", baseType: !80, size: 32, align: 32)
!136 = !DIDerivedType(tag: DW_TAG_member, name: "fill", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/937f}", baseType: !77, size: 32, align: 32, offset: 32)
!137 = !DIDerivedType(tag: DW_TAG_member, name: "align", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/937f}", baseType: !12, size: 8, align: 8, offset: 64)
!138 = !DIDerivedType(tag: DW_TAG_member, name: "width", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/937f}", baseType: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/7eef<{usize},>}", size: 128, align: 64, offset: 128)
!139 = !DIDerivedType(tag: DW_TAG_member, name: "precision", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/937f}", baseType: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/7eef<{usize},>}", size: 128, align: 64, offset: 256)
!140 = !DIDerivedType(tag: DW_TAG_member, name: "buf", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/937f}", baseType: !"{&mut{trait core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9358}}", size: 128, align: 64, offset: 384)
!141 = !DIDerivedType(tag: DW_TAG_member, name: "curarg", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/937f}", baseType: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/83a5<{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/938a},>}", size: 128, align: 64, offset: 512)
!142 = !DIDerivedType(tag: DW_TAG_member, name: "args", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/937f}", baseType: !"{&{[]{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/938a}}}", size: 128, align: 64, offset: 640)
!143 = !DICompositeType(tag: DW_TAG_union_type, name: "Option<usize>", scope: !31, file: !53, size: 128, align: 64, elements: !144, identifier: "{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/7eef<{usize},>}")
!144 = !{!145, !146}
!145 = !DIDerivedType(tag: DW_TAG_member, scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/7eef<{usize},>}", baseType: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/7eef<{usize},>}::None", size: 64, align: 64)
!146 = !DIDerivedType(tag: DW_TAG_member, scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/7eef<{usize},>}", baseType: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/7eef<{usize},>}::Some", size: 128, align: 64)
!147 = !DICompositeType(tag: DW_TAG_structure_type, name: "None", scope: !31, size: 64, align: 64, elements: !148, identifier: "{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/7eef<{usize},>}::None")
!148 = !{!149}
!149 = !DIDerivedType(tag: DW_TAG_member, name: "RUST$ENUM$DISR", scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/7eef<{usize},>}::None", baseType: !30, size: 64, align: 64)
!150 = !DICompositeType(tag: DW_TAG_structure_type, name: "Some", scope: !31, size: 128, align: 64, elements: !151, identifier: "{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/7eef<{usize},>}::Some")
!151 = !{!152, !153}
!152 = !DIDerivedType(tag: DW_TAG_member, name: "RUST$ENUM$DISR", scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/7eef<{usize},>}::Some", baseType: !30, size: 64, align: 64)
!153 = !DIDerivedType(tag: DW_TAG_member, name: "__0", scope: !"{enum core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/7eef<{usize},>}::Some", baseType: !46, size: 64, align: 64, offset: 64)
!154 = !DICompositeType(tag: DW_TAG_structure_type, name: "&mut Write", scope: !6, size: 128, align: 64, elements: !122, identifier: "{&mut{trait core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/9358}}")
!155 = !DICompositeType(tag: DW_TAG_structure_type, name: "Iter<core::fmt::ArgumentV1>", scope: !156, size: 128, align: 64, elements: !157, identifier: "{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/83a5<{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/938a},>}")
!156 = !DINamespace(name: "slice", scope: !7)
!157 = !{!158, !160, !161}
!158 = !DIDerivedType(tag: DW_TAG_member, name: "ptr", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/83a5<{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/938a},>}", baseType: !159, size: 64, align: 64)
!159 = !DIDerivedType(tag: DW_TAG_pointer_type, name: "*const core::fmt::ArgumentV1", baseType: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/938a}", size: 64, align: 64)
!160 = !DIDerivedType(tag: DW_TAG_member, name: "end", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/83a5<{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/938a},>}", baseType: !159, size: 64, align: 64, offset: 64)
!161 = !DIDerivedType(tag: DW_TAG_member, name: "_marker", scope: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/83a5<{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/938a},>}", baseType: !"{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/34d1<{&{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/938a}},>}", align: 8, offset: 128)
!162 = !DICompositeType(tag: DW_TAG_structure_type, name: "PhantomData<&core::fmt::ArgumentV1>", scope: !163, align: 8, elements: !122, identifier: "{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/34d1<{&{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/938a}},>}")
!163 = !DINamespace(name: "marker", scope: !7)
!164 = !DICompositeType(tag: DW_TAG_structure_type, name: "&[core::fmt::ArgumentV1]", size: 128, align: 64, elements: !165, identifier: "{&{[]{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/938a}}}")
!165 = !{!166, !167}
!166 = !DIDerivedType(tag: DW_TAG_member, name: "data_ptr", scope: !"{&{[]{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/938a}}}", baseType: !159, size: 64, align: 64)
!167 = !DIDerivedType(tag: DW_TAG_member, name: "length", scope: !"{&{[]{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/938a}}}", baseType: !46, size: 64, align: 64, offset: 64)
!168 = !{!169, !181, !187, !198, !226, !234, !245}
!169 = distinct !DISubprogram(name: "rdtscp_unsafe", linkageName: "_ZN11rdtscp_test13rdtscp_unsafeE", scope: !171, file: !170, line: 3, type: !172, isLocal: true, isDefinition: true, scopeLine: 3, flags: DIFlagPrototyped, isOptimized: true, templateParams: !122, variables: !174)
!170 = !DIFile(filename: "src/main.rs", directory: "/home/apanda/e2d2/debugging/rdtscp-test")
!171 = !DINamespace(name: "rdtscp_test", scope: null)
!172 = !DISubroutineType(types: !173)
!173 = !{!8}
!174 = !{!175, !177, !178, !179}
!175 = !DILocalVariable(name: "high", scope: !176, file: !170, line: 4, type: !80)
!176 = distinct !DILexicalBlock(scope: !169, file: !170, line: 3, column: 26)
!177 = !DILocalVariable(name: "low", scope: !176, file: !170, line: 5, type: !80)
!178 = !DILocalVariable(name: "aux", scope: !176, file: !170, line: 6, type: !80)
!179 = !DILocalVariable(name: "ret", scope: !180, file: !170, line: 13, type: !8)
!180 = distinct !DILexicalBlock(scope: !176, file: !170, line: 7, column: 4)
!181 = distinct !DISubprogram(name: "rdtsc_unsafe", linkageName: "_ZN11rdtscp_test12rdtsc_unsafeE", scope: !171, file: !170, line: 19, type: !172, isLocal: true, isDefinition: true, scopeLine: 19, flags: DIFlagPrototyped, isOptimized: true, templateParams: !122, variables: !182)
!182 = !{!183, !186}
!183 = !DILocalVariable(name: "low", scope: !184, file: !170, line: 21, type: !80)
!184 = distinct !DILexicalBlock(scope: !185, file: !170, line: 20, column: 4)
!185 = distinct !DILexicalBlock(scope: !181, file: !170, line: 19, column: 25)
!186 = !DILocalVariable(name: "high", scope: !184, file: !170, line: 22, type: !80)
!187 = distinct !DISubprogram(name: "cpuid", linkageName: "_ZN11rdtscp_test5cpuidE", scope: !171, file: !170, line: 33, type: !188, isLocal: true, isDefinition: true, scopeLine: 33, flags: DIFlagPrototyped, isOptimized: true, templateParams: !122, variables: !190)
!188 = !DISubroutineType(types: !189)
!189 = !{null}
!190 = !{!191, !195, !196, !197}
!191 = !DILocalVariable(name: "x1", scope: !192, file: !170, line: 35, type: !194)
!192 = distinct !DILexicalBlock(scope: !193, file: !170, line: 34, column: 4)
!193 = distinct !DILexicalBlock(scope: !187, file: !170, line: 33, column: 11)
!194 = !DIBasicType(name: "i32", size: 32, align: 32, encoding: DW_ATE_signed)
!195 = !DILocalVariable(name: "x2", scope: !192, file: !170, line: 36, type: !194)
!196 = !DILocalVariable(name: "x3", scope: !192, file: !170, line: 37, type: !194)
!197 = !DILocalVariable(name: "x4", scope: !192, file: !170, line: 38, type: !194)
!198 = distinct !DISubprogram(name: "main", linkageName: "_ZN11rdtscp_test4mainE", scope: !171, file: !170, line: 49, type: !188, isLocal: true, isDefinition: true, scopeLine: 49, flags: DIFlagPrototyped, isOptimized: true, templateParams: !122, variables: !199)
!199 = !{!200, !202, !203, !204, !205, !206, !208, !209, !210, !211, !212, !217, !219, !220, !221, !222, !223, !224, !225}
!200 = !DILocalVariable(name: "a", scope: !201, file: !170, line: 50, type: !8)
!201 = distinct !DILexicalBlock(scope: !198, file: !170, line: 49, column: 10)
!202 = !DILocalVariable(name: "b", scope: !201, file: !170, line: 51, type: !8)
!203 = !DILocalVariable(name: "c", scope: !201, file: !170, line: 52, type: !8)
!204 = !DILocalVariable(name: "d", scope: !201, file: !170, line: 53, type: !194)
!205 = !DILocalVariable(name: "e", scope: !201, file: !170, line: 54, type: !194)
!206 = !DILocalVariable(name: "e", scope: !207, file: !170, line: 55, type: !8)
!207 = distinct !DILexicalBlock(scope: !201, file: !170, line: 55, column: 8)
!208 = !DILocalVariable(name: "f", scope: !207, file: !170, line: 56, type: !194)
!209 = !DILocalVariable(name: "g", scope: !207, file: !170, line: 57, type: !8)
!210 = !DILocalVariable(name: "h", scope: !207, file: !170, line: 58, type: !8)
!211 = !DILocalVariable(name: "i", scope: !207, file: !170, line: 59, type: !8)
!212 = !DILocalVariable(name: "__arg5", scope: !213, file: !170, line: 1, type: !216)
!213 = distinct !DILexicalBlock(scope: !215, file: !214, line: 3, column: 10)
!214 = !DIFile(filename: "<std macros>", directory: "/home/apanda/e2d2/debugging/rdtscp-test")
!215 = distinct !DILexicalBlock(scope: !207, file: !170, line: 60, column: 9)
!216 = !DIDerivedType(tag: DW_TAG_pointer_type, name: "&i32", baseType: !194, size: 64, align: 64)
!217 = !DILocalVariable(name: "__arg1", scope: !213, file: !170, line: 1, type: !218)
!218 = !DIDerivedType(tag: DW_TAG_pointer_type, name: "&u64", baseType: !8, size: 64, align: 64)
!219 = !DILocalVariable(name: "__arg8", scope: !213, file: !170, line: 1, type: !218)
!220 = !DILocalVariable(name: "__arg4", scope: !213, file: !170, line: 1, type: !218)
!221 = !DILocalVariable(name: "__arg7", scope: !213, file: !170, line: 1, type: !218)
!222 = !DILocalVariable(name: "__arg6", scope: !213, file: !170, line: 1, type: !218)
!223 = !DILocalVariable(name: "__arg3", scope: !213, file: !170, line: 1, type: !216)
!224 = !DILocalVariable(name: "__arg2", scope: !213, file: !170, line: 1, type: !218)
!225 = !DILocalVariable(name: "__arg0", scope: !213, file: !170, line: 1, type: !218)
!226 = distinct !DISubprogram(name: "new_v1", linkageName: "_ZN4core3fmt8{{impl}}6new_v1E", scope: !228, file: !227, line: 243, type: !229, isLocal: false, isDefinition: true, scopeLine: 243, flags: DIFlagPrototyped, isOptimized: true, templateParams: !122, variables: !231)
!227 = !DIFile(filename: "src/libcore/fmt/mod.rs", directory: "/home/apanda/e2d2/debugging/rdtscp-test")
!228 = !DINamespace(name: "{{impl}}", scope: !6)
!229 = !DISubroutineType(types: !230)
!230 = !{!36, !47, !164}
!231 = !{!232, !233}
!232 = !DILocalVariable(name: "pieces", arg: 1, scope: !226, file: !227, line: 243, type: !"{&{[]{&{str}}}}")
!233 = !DILocalVariable(name: "args", arg: 2, scope: !226, file: !227, line: 244, type: !"{&{[]{struct core/9a0e4c7da41e93c2dd6633432bac65bcbcb58b4b8218b678c39d58c51d24a412/938a}}}")
!234 = distinct !DISubprogram(name: "new<u64>", linkageName: "_ZN4core3fmt8{{impl}}8new<u64>E", scope: !228, file: !227, line: 206, type: !235, isLocal: false, isDefinition: true, scopeLine: 206, flags: DIFlagPrototyped, isOptimized: true, templateParams: !240, variables: !242)
!235 = !DISubroutineType(types: !236)
!236 = !{!108, !218, !237}
!237 = !DIDerivedType(tag: DW_TAG_pointer_type, name: "fn(&u64, &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error>", baseType: !238, size: 64, align: 64)
!238 = !DISubroutineType(types: !239)
!239 = !{!116, !218, !120}
!240 = !{!241}
!241 = !DITemplateTypeParameter(name: "T", type: !8)
!242 = !{!243, !244}
!243 = !DILocalVariable(name: "x", arg: 1, scope: !234, file: !227, line: 206, type: !218)
!244 = !DILocalVariable(name: "f", arg: 2, scope: !234, file: !227, line: 207, type: !237)
!245 = distinct !DISubprogram(name: "new<i32>", linkageName: "_ZN4core3fmt8{{impl}}8new<i32>E", scope: !228, file: !227, line: 206, type: !246, isLocal: false, isDefinition: true, scopeLine: 206, flags: DIFlagPrototyped, isOptimized: true, templateParams: !251, variables: !253)
!246 = !DISubroutineType(types: !247)
!247 = !{!108, !216, !248}
!248 = !DIDerivedType(tag: DW_TAG_pointer_type, name: "fn(&i32, &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error>", baseType: !249, size: 64, align: 64)
!249 = !DISubroutineType(types: !250)
!250 = !{!116, !216, !120}
!251 = !{!252}
!252 = !DITemplateTypeParameter(name: "T", type: !194)
!253 = !{!254, !255}
!254 = !DILocalVariable(name: "x", arg: 1, scope: !245, file: !227, line: 206, type: !216)
!255 = !DILocalVariable(name: "f", arg: 2, scope: !245, file: !227, line: 207, type: !248)
!256 = !{!257}
!257 = !DIGlobalVariable(name: "__STATIC_FMTSTR", linkageName: "_ZN11rdtscp_test4main15__STATIC_FMTSTRE", scope: !258, file: !214, line: 3, type: !"{&{[]{&{str}}}}", isLocal: true, isDefinition: true)
!258 = !DINamespace(name: "main", scope: !171, file: !170, line: 49)
!259 = !{i32 2, !"Debug Info Version", i32 3}
!260 = !DIExpression()
!261 = !DILocation(line: 50, scope: !201)
!262 = !DILocation(line: 51, scope: !201)
!263 = !DILocation(line: 52, scope: !201)
!264 = !DILocation(line: 53, scope: !201)
!265 = !DILocation(line: 54, scope: !201)
!266 = !DILocation(line: 55, scope: !207)
!267 = !DILocation(line: 56, scope: !207)
!268 = !DILocation(line: 57, scope: !207)
!269 = !DILocation(line: 58, scope: !207)
!270 = !DILocation(line: 59, scope: !207)
!271 = !DILocation(line: 3, scope: !213)
!272 = !DILocation(line: 2, scope: !213)
!273 = !DILocation(line: 211, scope: !274, inlinedAt: !276)
!274 = distinct !DILexicalBlock(scope: !275, file: !227, line: 208, column: 8)
!275 = distinct !DILexicalBlock(scope: !234, file: !227, line: 207, column: 77)
!276 = distinct !DILocation(line: 2, scope: !213)
!277 = !DILocation(line: 211, scope: !274, inlinedAt: !278)
!278 = distinct !DILocation(line: 2, scope: !213)
!279 = !DILocation(line: 211, scope: !274, inlinedAt: !280)
!280 = distinct !DILocation(line: 2, scope: !213)
!281 = !DILocation(line: 211, scope: !282, inlinedAt: !284)
!282 = distinct !DILexicalBlock(scope: !283, file: !227, line: 208, column: 8)
!283 = distinct !DILexicalBlock(scope: !245, file: !227, line: 207, column: 77)
!284 = distinct !DILocation(line: 2, scope: !213)
!285 = !DILocation(line: 211, scope: !274, inlinedAt: !286)
!286 = distinct !DILocation(line: 2, scope: !213)
!287 = !DILocation(line: 211, scope: !282, inlinedAt: !288)
!288 = distinct !DILocation(line: 2, scope: !213)
!289 = !DILocation(line: 211, scope: !274, inlinedAt: !290)
!290 = distinct !DILocation(line: 2, scope: !213)
!291 = !DILocation(line: 211, scope: !274, inlinedAt: !292)
!292 = distinct !DILocation(line: 2, scope: !213)
!293 = !DILocation(line: 211, scope: !274, inlinedAt: !294)
!294 = distinct !DILocation(line: 2, scope: !213)
!295 = !DILocation(line: 246, scope: !296, inlinedAt: !297)
!296 = distinct !DILexicalBlock(scope: !226, file: !227, line: 244, column: 63)
!297 = distinct !DILocation(line: 2, scope: !215)
!298 = !DILocation(line: 2, scope: !215)
!299 = !DILocation(line: 247, scope: !296, inlinedAt: !297)
!300 = !DILocation(line: 248, scope: !296, inlinedAt: !297)
!301 = !DILocation(line: 60, scope: !207)
!302 = !DILocation(line: 4, scope: !176, inlinedAt: !303)
!303 = distinct !DILocation(line: 61, scope: !215)
!304 = !DILocation(line: 5, scope: !176, inlinedAt: !303)
!305 = !DILocation(line: 6, scope: !176, inlinedAt: !303)
!306 = !DILocation(line: 9, scope: !180, inlinedAt: !303)
!307 = !{i32 1}
!308 = !DILocation(line: 13, scope: !180, inlinedAt: !303)
!309 = !DILocation(line: 61, scope: !215)
!310 = !DILocation(line: 62, scope: !215)
!311 = !DILocation(line: 4, scope: !176, inlinedAt: !312)
!312 = distinct !DILocation(line: 63, scope: !313)
!313 = distinct !DILexicalBlock(scope: !215, file: !170, line: 62, column: 26)
!314 = !DILocation(line: 65, scope: !215)
!315 = !DILocation(line: 4, scope: !176, inlinedAt: !316)
!316 = distinct !DILocation(line: 66, scope: !317)
!317 = distinct !DILexicalBlock(scope: !215, file: !170, line: 65, column: 30)
!318 = !DILocation(line: 5, scope: !176, inlinedAt: !312)
!319 = !DILocation(line: 6, scope: !176, inlinedAt: !312)
!320 = !DILocation(line: 9, scope: !180, inlinedAt: !312)
!321 = !DILocation(line: 13, scope: !180, inlinedAt: !312)
!322 = !DILocation(line: 63, scope: !313)
!323 = !DILocation(line: 68, scope: !215)
!324 = !DILocation(line: 69, scope: !215)
!325 = !DILocation(line: 35, scope: !192, inlinedAt: !326)
!326 = distinct !DILocation(line: 70, scope: !215)
!327 = !DILocation(line: 36, scope: !192, inlinedAt: !326)
!328 = !DILocation(line: 37, scope: !192, inlinedAt: !326)
!329 = !DILocation(line: 38, scope: !192, inlinedAt: !326)
!330 = !DILocation(line: 39, scope: !192, inlinedAt: !326)
!331 = !{i32 5}
!332 = !DILocation(line: 40, scope: !192, inlinedAt: !326)
!333 = !{i32 7}
!334 = !DILocation(line: 42, scope: !192, inlinedAt: !326)
!335 = !{i32 9}
!336 = !DILocation(line: 4, scope: !176, inlinedAt: !337)
!337 = distinct !DILocation(line: 71, scope: !215)
!338 = !DILocation(line: 5, scope: !176, inlinedAt: !337)
!339 = !DILocation(line: 6, scope: !176, inlinedAt: !337)
!340 = !DILocation(line: 9, scope: !180, inlinedAt: !337)
!341 = !DILocation(line: 13, scope: !180, inlinedAt: !337)
!342 = !DILocation(line: 71, scope: !215)
!343 = !DILocation(line: 35, scope: !192, inlinedAt: !344)
!344 = distinct !DILocation(line: 72, scope: !215)
!345 = !DILocation(line: 36, scope: !192, inlinedAt: !344)
!346 = !DILocation(line: 37, scope: !192, inlinedAt: !344)
!347 = !DILocation(line: 38, scope: !192, inlinedAt: !344)
!348 = !DILocation(line: 39, scope: !192, inlinedAt: !344)
!349 = !DILocation(line: 40, scope: !192, inlinedAt: !344)
!350 = !DILocation(line: 42, scope: !192, inlinedAt: !344)
!351 = !DILocation(line: 73, scope: !215)
!352 = !DILocation(line: 74, scope: !215)
!353 = !DILocation(line: 1, scope: !213)
!354 = !DIExpression(DW_OP_deref)
!355 = !DILocation(line: 207, scope: !234, inlinedAt: !276)
!356 = !DILocation(line: 210, scope: !274, inlinedAt: !276)
!357 = !{!358}
!358 = distinct !{!358, !359, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE: argument 0"}
!359 = distinct !{!359, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE"}
!360 = !{!361}
!361 = distinct !{!361, !359, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE: argument 1"}
!362 = !DILocation(line: 207, scope: !234, inlinedAt: !278)
!363 = !DILocation(line: 210, scope: !274, inlinedAt: !278)
!364 = !{!365}
!365 = distinct !{!365, !366, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE: argument 0"}
!366 = distinct !{!366, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE"}
!367 = !{!368}
!368 = distinct !{!368, !366, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE: argument 1"}
!369 = !DILocation(line: 207, scope: !234, inlinedAt: !280)
!370 = !DILocation(line: 210, scope: !274, inlinedAt: !280)
!371 = !{!372}
!372 = distinct !{!372, !373, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE: argument 0"}
!373 = distinct !{!373, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE"}
!374 = !{!375}
!375 = distinct !{!375, !373, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE: argument 1"}
!376 = !DILocation(line: 207, scope: !245, inlinedAt: !284)
!377 = !DILocation(line: 210, scope: !282, inlinedAt: !284)
!378 = !{!379}
!379 = distinct !{!379, !380, !"_ZN4core3fmt10ArgumentV13new17h0fd593fc09de7f27E: argument 0"}
!380 = distinct !{!380, !"_ZN4core3fmt10ArgumentV13new17h0fd593fc09de7f27E"}
!381 = !{!382}
!382 = distinct !{!382, !380, !"_ZN4core3fmt10ArgumentV13new17h0fd593fc09de7f27E: argument 1"}
!383 = !DILocation(line: 207, scope: !234, inlinedAt: !286)
!384 = !DILocation(line: 210, scope: !274, inlinedAt: !286)
!385 = !{!386}
!386 = distinct !{!386, !387, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE: argument 0"}
!387 = distinct !{!387, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE"}
!388 = !{!389}
!389 = distinct !{!389, !387, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE: argument 1"}
!390 = !DILocation(line: 207, scope: !245, inlinedAt: !288)
!391 = !DILocation(line: 210, scope: !282, inlinedAt: !288)
!392 = !{!393}
!393 = distinct !{!393, !394, !"_ZN4core3fmt10ArgumentV13new17h0fd593fc09de7f27E: argument 0"}
!394 = distinct !{!394, !"_ZN4core3fmt10ArgumentV13new17h0fd593fc09de7f27E"}
!395 = !{!396}
!396 = distinct !{!396, !394, !"_ZN4core3fmt10ArgumentV13new17h0fd593fc09de7f27E: argument 1"}
!397 = !DILocation(line: 207, scope: !234, inlinedAt: !290)
!398 = !DILocation(line: 210, scope: !274, inlinedAt: !290)
!399 = !{!400}
!400 = distinct !{!400, !401, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE: argument 0"}
!401 = distinct !{!401, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE"}
!402 = !{!403}
!403 = distinct !{!403, !401, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE: argument 1"}
!404 = !DILocation(line: 207, scope: !234, inlinedAt: !292)
!405 = !DILocation(line: 210, scope: !274, inlinedAt: !292)
!406 = !{!407}
!407 = distinct !{!407, !408, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE: argument 0"}
!408 = distinct !{!408, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE"}
!409 = !{!410}
!410 = distinct !{!410, !408, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE: argument 1"}
!411 = !DILocation(line: 207, scope: !234, inlinedAt: !294)
!412 = !DILocation(line: 210, scope: !274, inlinedAt: !294)
!413 = !{!414}
!414 = distinct !{!414, !415, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE: argument 0"}
!415 = distinct !{!415, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE"}
!416 = !{!417}
!417 = distinct !{!417, !415, !"_ZN4core3fmt10ArgumentV13new17h861f445cfaa7d89fE: argument 1"}
!418 = !DIExpression(DW_OP_bit_piece, 0, 64)
!419 = !DILocation(line: 243, scope: !226, inlinedAt: !297)
!420 = !DIExpression(DW_OP_bit_piece, 64, 64)
!421 = !DILocation(line: 244, scope: !226, inlinedAt: !297)
!422 = !{!423}
!423 = distinct !{!423, !424, !"_ZN4core3fmt9Arguments6new_v117h00bc5e947a058f1aE: argument 0"}
!424 = distinct !{!424, !"_ZN4core3fmt9Arguments6new_v117h00bc5e947a058f1aE"}
!425 = !{!426}
!426 = distinct !{!426, !424, !"_ZN4core3fmt9Arguments6new_v117h00bc5e947a058f1aE: argument 1"}
!427 = !DILocation(line: 5, scope: !176, inlinedAt: !316)
!428 = !DILocation(line: 6, scope: !176, inlinedAt: !316)
!429 = !DILocation(line: 9, scope: !180, inlinedAt: !316)
!430 = !DILocation(line: 13, scope: !180, inlinedAt: !316)
!431 = !DILocation(line: 66, scope: !317)
