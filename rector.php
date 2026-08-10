<?php

declare(strict_types=1);

use Rector\CodeQuality\Rector\BooleanAnd\SimplifyEmptyArrayCheckRector;
use Rector\CodeQuality\Rector\Catch_\ThrowWithPreviousExceptionRector;
use Rector\CodeQuality\Rector\Empty_\SimplifyEmptyCheckOnEmptyArrayRector;
use Rector\CodeQuality\Rector\Expression\TernaryFalseExpressionToIfRector;
use Rector\CodeQuality\Rector\FunctionLike\SimplifyUselessVariableRector;
use Rector\CodeQuality\Rector\Identical\FlipTypeControlToUseExclusiveTypeRector;
use Rector\CodeQuality\Rector\Identical\SimplifyConditionsRector;
use Rector\CodeQuality\Rector\Identical\StrlenZeroToIdenticalEmptyStringRector;
use Rector\CodeQuality\Rector\NullsafeMethodCall\CleanupUnneededNullsafeOperatorRector;
use Rector\CodeQuality\Rector\Ternary\SwitchNegatedTernaryRector;
use Rector\CodeQuality\Rector\Ternary\UnnecessaryTernaryExpressionRector;
use Rector\CodingStyle\Rector\String_\SimplifyQuoteEscapeRector;
use Rector\Config\RectorConfig;
use Rector\DeadCode\Rector\If_\ReduceAlwaysFalseIfOrRector;
use Rector\DeadCode\Rector\If_\RemoveAlwaysTrueIfConditionRector;
use Rector\EarlyReturn\Rector\If_\ChangeIfElseValueAssignToEarlyReturnRector;
use Rector\Instanceof_\Rector\Ternary\FlipNegatedTernaryInstanceofRector;
use Rector\Php54\Rector\Array_\LongArrayToShortArrayRector;
use Rector\Php70\Rector\Ternary\TernaryToNullCoalescingRector;
use Rector\Php74\Rector\Closure\ClosureToArrowFunctionRector;
use Rector\TypeDeclaration\Rector\StmtsAwareInterface\DeclareStrictTypesRector;

return RectorConfig::configure()
    ->withRules(
        [
            DeclareStrictTypesRector::class,
            CleanupUnneededNullsafeOperatorRector::class,
            FlipTypeControlToUseExclusiveTypeRector::class,
            SimplifyConditionsRector::class,
            SimplifyEmptyArrayCheckRector::class,
            SimplifyEmptyCheckOnEmptyArrayRector::class,
            SimplifyUselessVariableRector::class,
            StrlenZeroToIdenticalEmptyStringRector::class,
            SwitchNegatedTernaryRector::class,
            TernaryFalseExpressionToIfRector::class,
            TernaryToNullCoalescingRector::class,
            ThrowWithPreviousExceptionRector::class,
            UnnecessaryTernaryExpressionRector::class,
            SimplifyQuoteEscapeRector::class,
            ReduceAlwaysFalseIfOrRector::class,
            RemoveAlwaysTrueIfConditionRector::class,
            ClosureToArrowFunctionRector::class,
            ChangeIfElseValueAssignToEarlyReturnRector::class,
            FlipNegatedTernaryInstanceofRector::class,
            LongArrayToShortArrayRector::class,
        ]
    )
    ->withPaths([
        __DIR__.'/app',
        __DIR__.'/database',
        __DIR__.'/config',
    ])
    ->withTypeCoverageLevel(0);
