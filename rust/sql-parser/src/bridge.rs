// Copyright (c) 2026 OceanBase.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    grammar,
    node::{Kind, Node},
    SqlMode,
};

#[cxx::bridge(namespace = "oceanbase::sql::peg")]
pub(crate) mod ffi {
    #[derive(Clone, Copy, Default)]
    struct SqlMode {
        ansi_quotes: bool,
        no_backslash_escapes: bool,
        pipes_as_concat: bool,
        high_not_precedence: bool,
    }

    struct ScanOptions {
        mode: SqlMode,
        parameterize: bool,
        filter_hints: bool,
        format: bool,
        batch_statement: bool,
    }

    enum LiteralKind {
        Integer,
        Unsigned,
        Decimal,
        Float,
        String,
        NationalString,
        Binary,
        Boolean,
        Null,
        Date,
        Time,
        Timestamp,
        Placeholder,
    }

    struct Literal {
        kind: LiteralKind,
        start: usize,
        end: usize,
        sql_offset: usize,
        value: u64,
        content_start: usize,
        content_end: usize,
        concatenated: bool,
        first_part_length: usize,
    }

    struct ValuesPosition {
        sql_offset: usize,
        literal_index: usize,
    }

    struct ScanResult {
        sql: String,
        literals: Vec<Literal>,
        literal_text: Vec<u8>,
        parameter_names: Vec<String>,
        ordinal_parameters: usize,
        values: Vec<ValuesPosition>,
        insert_values_offset: usize,
        error: String,
        error_offset: usize,
    }

    enum NodeKind {
        Missing,
        EmptyQuery,
        ExplainIntoTable,
        ExplainStatementId,
        Having,
        QueryLock,
        TrailingIntoVariables,
        InsertTarget,
        InsertColumns,
        InsertRows,
        InsertQuery,
        InsertAssignments,
        DuplicateKeyAssignments,
        DeleteTargets,
        TableAlias,
        DerivedColumns,
        Lateral,
        ParenthesizedTable,
        JoinOn,
        HintQbGroups,
        VectorParameter,
        CallName,
        CallArguments,

        Select,
        WithClause,
        CommonTable,
        SelectOptions,
        FoundRows,
        StraightJoin,
        Union,
        UnionAll,
        Intersect,
        Except,
        Explain,
        ExplainBasic,
        ExplainOutline,
        ExplainExtended,
        ExplainExtendedNoAddr,
        ExplainPartitions,
        ExplainPlanRegress,
        ExplainTraditional,
        ExplainJson,
        ExplainPretty,
        ExplainPrettyColor,
        HashAggregate,
        NoHashAggregate,
        Parallel,
        TableParallel,
        DmlParallel,
        NoParallel,
        EnableParallelDml,
        DisableParallelDml,
        QueryTimeout,
        TopK,
        DynamicSampling,
        TableDynamicSampling,
        Snapshot,
        Sample,
        SampleInt,
        SampleDecimal,
        SampleBase,
        SampleIncremental,
        TableOptions,
        Organization,
        OrganizationIndex,
        OrganizationHeap,
        TableCharset,
        Collation,
        SetCollation,
        Compression,
        TableMode,
        StoreFormat,
        TableAutoIncrement,
        BlockSize,
        TabletSize,
        LobInrowThreshold,
        PctFree,
        Index,
        IndexColumns,
        IndexColumn,
        ParserName,
        ParserProperties,
        ParserMinTokenSize,
        ParserMaxTokenSize,
        ParserNgramTokenSize,
        ParserStopwordTable,
        ParserDictTable,
        ParserQuantifierTable,
        ParserIkMode,
        ParserMinNgramSize,
        ParserMaxNgramSize,
        MatchColumns,
        MatchAgainst,
        IndexScope,
        StoringColumns,
        UsingHash,
        CreateIndex,
        DropIndex,
        PartitionOption,
        UsePartition,
        Names,
        TableIndexHints,
        TableIndexHint,
        Use,
        Force,
        KeyPartition,
        HashPartition,
        ListPartition,
        ListColumnsPartition,
        RangePartition,
        RangeColumnsPartition,
        Partitions,
        HashSubpartitions,
        RangeSubpartitions,
        ListSubpartitions,
        HashPartitionElement,
        ListPartitionElement,
        RangePartitionElement,
        MaxValue,
        PartitionKeyFunction,
        PartitionHashFunction,
        ProjectList,
        Projection,
        Alias,
        StringAlias,
        FromList,
        Relation,
        Table,
        Ident,
        Column,
        Star,
        Int,
        Uint,
        Number,
        Double,
        String,
        StringColumnName,
        StringConcat,
        CollectionType,
        ArrayLiteral,
        MapLiteral,
        ArrayAgg,
        ArrayMap,
        ArrayFilter,
        ArrayFirst,
        ArraySortBy,
        Lambda,
        VectorDistance,
        VectorSimilarity,
        Approximate,
        VectorParameters,
        CharsetLiteral,
        CharsetName,
        ConvertTableCharset,
        TableCollection,
        DateLiteral,
        TimeLiteral,
        TimestampLiteral,
        Bool,
        Null,
        Parameter,
        Parenthesized,
        Unary,
        Binary,
        Where,
        Order,
        SortList,
        SortKey,
        Asc,
        Desc,
        Limit,
        CommaLimit,
        Distinct,
        ExprList,
        Between,
        In,
        Is,
        Function,
        Cast,
        If,
        IsNull,
        CurrentTimestamp,
        CurrentTime,
        CurrentDate,
        UtcTimestamp,
        UtcTime,
        UtcDate,
        Sysdate,
        CurrentUser,
        Count,
        Sum,
        Min,
        Max,
        Avg,
        JsonArrayAgg,
        JsonObjectAgg,
        Stddev,
        StddevPop,
        StddevSamp,
        Variance,
        VarPop,
        VarSamp,
        BitAnd,
        BitOr,
        BitXor,
        All,
        Any,
        Exists,
        JoinedTable,
        InnerJoin,
        LeftJoin,
        RightJoin,
        FullJoin,
        ColumnList,
        GroupBy,
        WithRollup,
        Rollup,
        DefaultInt,
        EnumType,
        SetType,
        StringList,
        BinaryAttribute,
        Insert,
        Replace,
        SingleTableInsert,
        InsertInto,
        ValueList,
        ValueVector,
        Empty,
        Default,
        Ignore,
        Assignment,
        AssignList,
        TableReferences,
        Update,
        Delete,
        DeleteTables,
        Begin,
        ConsistentSnapshot,
        ReadOnly,
        ReadWrite,
        SetTransaction,
        TransactionCharacteristics,
        Commit,
        Rollback,
        UseDatabase,
        CreateDatabase,
        DatabaseOptions,
        DropDatabase,
        IfExists,
        IfNotExists,
        CreateTable,
        CreateView,
        GroupConcat,
        Separator,
        GatherStatistics,
        NoGatherStatistics,
        DropView,
        ViewList,
        ShowCreateView,
        CreateTableLike,
        ForkTable,
        ForkDatabase,
        Charset,
        DropTable,
        TruncateTable,
        Temporary,
        Srid,
        Point,
        LineString,
        Polygon,
        MultiPoint,
        MultiLineString,
        MultiPolygon,
        GeometryCollection,
        OptimizeTable,
        PurgeRecyclebin,
        PurgeTable,
        PurgeIndex,
        PurgeDatabase,
        RenameTables,
        RenameTableAction,
        AlterTable,
        AlterCheckConstraint,
        AlterPartition,
        Repartition,
        PartitionAdd,
        PartitionExchange,
        PartitionDrop,
        PartitionTruncate,
        SubpartitionDrop,
        SubpartitionTruncate,
        RangePartitions,
        AlterTableActions,
        AlterTableOption,
        TableRename,
        AlterColumnOption,
        ColumnAdd,
        ColumnDrop,
        ColumnModify,
        ColumnChange,
        ColumnRename,
        ColumnAlter,
        ColumnFirst,
        ColumnBefore,
        ColumnAfter,
        AlterIndexOption,
        IndexAdd,
        IndexDrop,
        IndexRename,
        PrimaryKeyDrop,
        ShowIndexes,
        TableList,
        TableElements,
        ColumnDefinition,
        ColumnAttributes,
        CheckConstraint,
        CheckConstraintDrop,
        Enforced,
        NotEnforced,
        NotNull,
        Nullable,
        PrimaryKey,
        ColumnPrimaryKey,
        UniqueKey,
        AutoIncrement,
        ColumnDefault,
        OnUpdate,
        ForeignKey,
        ForeignKeyDrop,
        AlterForeignKey,
        DropConstraint,
        ReferenceOptions,
        ReferenceOption,
        Restrict,
        Cascade,
        SetNull,
        NoAction,
        SetDefault,
        ColumnComment,
        GeneratedColumn,
        Always,
        VirtualColumn,
        StoredColumn,
        DataType,
        SystemVariable,
        UserVariable,
        GetUserVariable,
        UserAssignment,
        UserVariableTarget,
        VariableSet,
        VariableValue,
        SetNames,
        SetCharset,
        ShowDatabases,
        ShowCharset,
        ShowCollation,
        ShowTables,
        ShowColumns,
        ShowVariables,
        ShowParameters,
        ShowProcesslist,
        ShowCreateTable,
        ShowCreateDatabase,
        ShowCreateProcedure,
        ShowCreateFunction,
        ShowCreateTrigger,
        ShowWarnings,
        ShowErrors,
        ShowLimit,
        LikeClause,
        Case,
        WhenList,
        When,
        DefaultNull,
        Savepoint,
        RollbackSavepoint,
        ReleaseSavepoint,
        LockWait,
        LockWaitDecimal,
        SkipLocked,
        HexString,
        BitString,
        CastType,
        SetConfig,
        Freeze,
        ResetConfig,
        FlushCache,
        FlushKvCache,
        CacheName,
        CacheType,
        CacheNamespace,
        SqlId,
        SchemaId,
        DatabaseList,
        ConfigList,
        ConfigAction,
        SetTracepoint,
        TracepointOptions,
        TracepointNumber,
        TracepointName,
        TracepointOccurrence,
        TracepointFrequency,
        TracepointError,
        TracepointMatch,
        Link,
        Values,
        ValuesRows,
        Hints,
        BeginOutline,
        EndOutline,
        HintQbNames,
        Materialize,
        CoalesceSubqueries,
        NoCoalesceSubqueries,
        HashSet,
        NoHashSet,
        PqDistribute,
        PqMap,
        PqSet,
        DistributeMethods,
        UseConcat,
        NoExpand,
        PqGroupBy,
        PqDistinct,
        GroupByPushdown,
        NoGroupByPushdown,
        HashDistinct,
        NoHashDistinct,
        DistinctPushdown,
        NoDistinctPushdown,
        DistributeAll,
        DistributeNone,
        DistributePartition,
        DistributeRandom,
        DistributeHash,
        DistributeBroadcast,
        DistributeLocal,
        DistributeBc2Host,
        DistributeRange,
        DistributeList,
        DistributeBasic,
        DistributeHashLocal,
        HintRelation,
        HintTables,
        JoinHintTables,
        Leading,
        UseNl,
        NoUseNl,
        UseNlMaterialization,
        NoUseNlMaterialization,
        UseHash,
        NoUseHash,
        UseMerge,
        NoUseMerge,
        QbName,
        OptParam,
        NoRewrite,
        MergeView,
        NoMergeView,
        OuterToInner,
        NoOuterToInner,
        Unnest,
        NoUnnest,
        SimplifyGroupBy,
        NoSimplifyGroupBy,
        PredicateDeduce,
        NoPredicateDeduce,
        ProjectPrune,
        NoProjectPrune,
        SemiToInner,
        NoSemiToInner,
        EliminateJoin,
        NoEliminateJoin,
        LeftToAnti,
        NoLeftToAnti,
        PlaceGroupBy,
        NoPlaceGroupBy,
        WindowMagic,
        NoWindowMagic,
        PushPredCte,
        NoPushPredCte,
        ReplaceConst,
        NoReplaceConst,
        SimplifyOrderBy,
        NoSimplifyOrderBy,
        SimplifyDistinct,
        NoSimplifyDistinct,
        SimplifyWinfunc,
        NoSimplifyWinfunc,
        SimplifyExpr,
        NoSimplifyExpr,
        SimplifyLimit,
        NoSimplifyLimit,
        SimplifySubquery,
        NoSimplifySubquery,
        FastMinmax,
        NoFastMinmax,
        SimplifySet,
        NoSimplifySet,
        PushLimit,
        NoPushLimit,
        PullupExpr,
        NoPullupExpr,
        AggrFirstUnnest,
        NoAggrFirstUnnest,
        JoinFirstUnnest,
        NoJoinFirstUnnest,
        Decorrelate,
        NoDecorrelate,
        UseLateMaterialization,
        NoUseLateMaterialization,
        TransformDistinctAgg,
        NoTransformDistinctAgg,
        PushSubq,
        NoPushSubq,
        Inline,
        UseDas,
        NoUseDas,
        UsePlanCache,
        WindowFunction,
        Window,
        WindowSpec,
        WindowFrame,
        WindowBound,
        WindowInterval,
        HiddenInt,
        HiddenDouble,
        HiddenString,
        JsonQuery,
        JsonValue,
        DateUnit,
        RowNumber,
        Rank,
        DenseRank,
        PercentRank,
        CumeDist,
        Ntile,
        FirstValue,
        LastValue,
        NthValue,
        Lead,
        Lag,
        First,
        Last,
        Respect,
        ReadConsistency,
        NamedWindows,
        NamedWindow,
        CreateUser,
        DropUser,
        CreateRole,
        DropRole,
        GrantRole,
        RevokeRole,
        WithAdminOption,
        WithGrantOption,
        IgnoreUnknownUser,
        SetRole,
        SetDefaultRole,
        ListedRoles,
        AllRoles,
        AllExceptRoles,
        NoRoles,
        DefaultRoles,
        AlterUserDefaultRole,
        Kill,
        Users,
        Roles,
        UserSpec,
        UserHost,
        Password,
        PlaintextAuthentication,
        EncodedAuthentication,
        AuthenticationPlugin,
        SetPassword,
        Privilege,
        Privileges,
        GlobalPrivilegeTarget,
        DatabasePrivilegeTarget,
        ObjectPrivilegeTarget,
        WildcardQualifiedObjectTarget,
        TargetDatabase,
        TargetObject,
        TablePrivilegeObject,
        ProcedurePrivilegeObject,
        FunctionPrivilegeObject,
        Grant,
        Revoke,
        RevokeAll,
        ShowGrants,
        Prepare,
        Execute,
        Deallocate,
        CreateProcedure,
        CreateTrigger,
        DropTrigger,
        TriggerSource,
        TriggerDefinition,
        TriggerOrder,
        CreateFunction,
        DropProcedure,
        DropFunction,
        AlterProcedure,
        AlterFunction,
        RoutineName,
        RoutineParameters,
        RoutineParameter,
        RowType,
        PlTypeAccess,
        RoutineClauses,
        Deterministic,
        DataAccess,
        InvokeRights,
        PlBlock,
        PlStatements,
        PlDeclarations,
        PlDeclaration,
        PlIdentifiers,
        PlDefault,
        PlHandler,
        PlHandlerConditions,
        PlConditionDeclaration,
        PlCondition,
        SqlState,
        SqlException,
        SqlWarning,
        SqlNotFound,
        PlSql,
        PlSubquery,
        PlIf,
        PlElse,
        PlCase,
        PlWhen,
        PlWhile,
        PlLoop,
        PlRepeat,
        PlReturn,
        PlLeave,
        PlIterate,
        PlLabeledBlock,
        PlLabeledControl,
        PlCall,
        PlCallName,
        PlCallArguments,
        PlNamedArgument,
        Diagnostics,
        PlSignal,
        PlResignal,
        PlSignalItems,
        PlSignalItem,
        PlCursor,
        PlOpen,
        PlClose,
        PlFetch,
        PlInto,
        IntoVariables,
        IntoVariableList,
        CreatePackage,
        CreatePackageBody,
        DropPackage,
        Package,
        PackageBody,
        PackageStatements,
        PackageBodyStatements,
        SubProcedureDeclaration,
        SubFunctionDeclaration,
        SubProcedureDefinition,
        SubFunctionDefinition,
        InterfacePragma,
        ObjectAccess,
        JsonTable,
        JsonColumn,
        HiddenNull,
        NullClause,
        Analyze,
        StatsOptions,
        StatsAll,
        StatsColumns,
        StatsColumn,
        StatsExtension,
        StatsBucketCount,
        DbmsStats,
        FullScan,
        IndexScan,
        IndexAsc,
        IndexDesc,
        ApproxCountDistinct,
        ApproxCountDistinctSynopsis,
        ApproxCountDistinctSynopsisMerge,
        SumOpnsize,
        TopFreHist,
        HybridHist,
    }

    struct Node {
        kind: NodeKind,
        children: Vec<Node>,
        text: String,
        start: usize,
        end: usize,
        value: u64,
        depth: usize,
    }

    enum TransactionAccess {
        Default,
        ReadOnly,
        ReadWrite,
    }

    struct TransactionOptions {
        consistent_snapshot: bool,
        access: TransactionAccess,
    }

    struct ParseTree {
        root: Node,
        expanded_sql: String,
        parameters: usize,
        parameter_names: Vec<String>,
        named_parameters: bool,
        error: String,
        error_offset: usize,
    }

    struct ColumnReference {
        parts: Vec<String>,
        wildcard: bool,
    }

    struct ExpressionInfo {
        root_kind: NodeKind,
        function_name: String,
        qualified_function: bool,
        columns: Vec<ColumnReference>,
        error: String,
        error_offset: usize,
    }

    struct StatementSplits {
        ends: Vec<usize>,
        failed: bool,
    }

    enum ParseMode {
        Sql,
        Routine,
        Trigger,
        Statistics,
        Expression,
        RoutineDefinition,
    }

    enum SearchNodeKind {
        Term,
        List,
        Group,
        Required,
        Excluded,
    }

    struct SearchNode {
        kind: SearchNodeKind,
        text: String,
        children: Vec<SearchNode>,
    }

    struct SearchTree {
        root: SearchNode,
        error: String,
    }

    enum AccountKind {
        Named,
        Current,
    }

    enum AccountNameForm {
        Identifier,
        String,
    }

    struct AccountIdentity<'a> {
        name: &'a str,
        host: &'a str,
        name_form: AccountNameForm,
        has_host: bool,
    }

    enum AuthenticationKind {
        Absent,
        Plaintext,
        Encoded,
    }

    enum RoleSelection {
        Listed,
        All,
        AllExcept,
        None,
        Default,
    }

    enum PrivilegeScope {
        Global,
        Database,
        Object,
        WildcardQualifiedObject,
    }

    enum PrivilegeObject {
        Unspecified,
        Table,
        Procedure,
        Function,
    }

    struct AccessOptions {
        if_exists: bool,
        if_not_exists: bool,
        grant_option: bool,
        admin_option: bool,
        ignore_unknown_user: bool,
    }

    struct PrivilegeTarget {
        scope: PrivilegeScope,
        object: PrivilegeObject,
        explicit_database: bool,
    }

    enum ExplainFormat {
        Default,
        Basic,
        Outline,
        Extended,
        ExtendedNoAddr,
        Partitions,
        PlanRegress,
        Traditional,
        Json,
    }
    enum ExplainDisplay {
        Default,
        Pretty,
        PrettyColor,
    }
    enum SetOperator {
        None,
        Union,
        UnionAll,
        Intersect,
        Except,
    }

    enum AliasForm {
        Absent,
        Identifier,
        String,
    }

    enum SortDirection {
        Unspecified,
        Ascending,
        Descending,
    }

    enum IntoPosition {
        Absent,
        AfterProjection,
        AfterQuery,
    }

    enum InsertSource {
        Rows,
        Query,
        Assignments,
    }

    enum LockKind {
        Wait,
        SkipLocked,
    }

    enum LockWait {
        Default,
        NoWait,
        Duration,
    }

    enum JoinKind {
        Inner,
        Left,
        Right,
        Full,
    }

    enum JoinCondition {
        None,
        On,
        Using,
    }

    enum SampleMethod {
        Row,
        Block,
        Hybrid,
    }

    enum SampleScope {
        All,
        Base,
        Incremental,
    }

    enum IndexHintKind {
        Use,
        Force,
        Ignore,
    }

    enum BinaryNotation {
        None,
        Hex,
        Bit,
    }

    struct ExpressionLiteral<'a> {
        kind: LiteralKind,
        integer: u64,
        text: &'a str,
        charset: &'a str,
        binary_notation: BinaryNotation,
        hidden: bool,
        concatenated: bool,
        first_part_length: usize,
        date_unit: &'a str,
    }

    enum UnaryOperator {
        Plus,
        Minus,
        LogicalNot,
        BitwiseNot,
    }

    enum BinaryOperator {
        Add,
        Subtract,
        Multiply,
        Divide,
        IntegerDivide,
        Modulo,
        BitOr,
        BitAnd,
        BitXor,
        ShiftLeft,
        ShiftRight,
        Concat,
        Equal,
        NullSafeEqual,
        NotEqual,
        Less,
        LessEqual,
        Greater,
        GreaterEqual,
        LogicalAnd,
        LogicalOr,
        LogicalXor,
        Like,
        NotLike,
        Regexp,
        NotRegexp,
    }

    enum MembershipSource {
        Expressions,
        Query,
    }

    enum ComparisonQuantifier {
        All,
        Any,
    }

    enum AggregateQuantifier {
        Default,
        All,
        Distinct,
    }

    enum VariableScope {
        Default,
        Global,
        Session,
    }

    enum TypeName {
        TinyInteger,
        SmallInteger,
        MediumInteger,
        Integer,
        BigInteger,
        Float,
        Double,
        Decimal,
        Boolean,
        Char,
        VarChar,
        Binary,
        VarBinary,
        TinyText,
        Text,
        MediumText,
        LongText,
        TinyBlob,
        Blob,
        MediumBlob,
        LongBlob,
        String,
        Date,
        Time,
        DateTime,
        Timestamp,
        Year,
        Json,
        Bit,
        Enum,
        Set,
        Geometry,
        Signed,
        Unsigned,
        Array,
        Vector,
        Map,
        SparseVector,
    }

    enum GeometryType {
        Geometry,
        Point,
        LineString,
        Polygon,
        MultiPoint,
        MultiLineString,
        MultiPolygon,
        GeometryCollection,
    }

    struct TypeSyntax<'a> {
        name: TypeName,
        spelling: &'a str,
        argument_count: usize,
        has_length: bool,
        length: u64,
        has_precision: bool,
        precision: u64,
        has_scale: bool,
        scale: u64,
        is_unsigned: bool,
        zerofill: bool,
        binary: bool,
        has_charset: bool,
        charset: &'a str,
        has_collation: bool,
        collation: &'a str,
        geometry: GeometryType,
    }

    enum NullTreatment {
        Unspecified,
        Respect,
        Ignore,
    }

    enum NthOrigin {
        Unspecified,
        First,
        Last,
    }

    enum FrameUnit {
        Rows,
        Range,
    }

    enum FrameBound {
        CurrentRow,
        UnboundedPreceding,
        UnboundedFollowing,
        Preceding,
        Following,
    }

    enum VectorMetric {
        Unspecified,
        Cosine,
        Dot,
        Euclidean,
        Manhattan,
    }

    enum FulltextMode {
        Natural,
        NaturalExpansion,
        Boolean,
        Expansion,
        Phrase,
    }

    enum JsonResponse {
        Unspecified,
        Error,
        Null,
        Default,
        Empty,
        EmptyArray,
        EmptyObject,
        Dot,
    }

    enum JsonScalars {
        Unspecified,
        Allow,
        Disallow,
    }

    enum JsonWrapper {
        Unspecified,
        Without,
        WithoutArray,
        With,
        WithArray,
        Unconditional,
        Conditional,
        UnconditionalArray,
        ConditionalArray,
    }

    enum JsonColumnKind {
        Ordinality,
        Exists,
        Value,
        Nested,
    }

    struct JsonOptions {
        truncate: bool,
        ascii: bool,
        pretty: bool,
        asis: bool,
        multivalue: bool,
        scalars: JsonScalars,
        wrapper: JsonWrapper,
        on_error: JsonResponse,
        on_empty: JsonResponse,
        on_mismatch: JsonResponse,
    }

    enum ParallelPolicy {
        Explicit,
        Manual,
        Automatic,
    }

    enum ReadConsistency {
        Weak,
        Strong,
        Frozen,
    }

    enum PlanCachePolicy {
        None,
        Default,
    }

    enum MergeDirection {
        Default,
        LessThan,
        GreaterThan,
    }

    enum RoutineDataAccess {
        ContainsSql,
        NoSql,
        ReadsSqlData,
        ModifiesSqlData,
    }

    enum ViewCheck {
        None,
        Local,
        Cascaded,
    }

    struct ViewDefinition<'a> {
        relation: &'a Node,
        columns: &'a [Node],
        query: &'a Node,
        replace: bool,
        alter: bool,
        check: ViewCheck,
    }

    enum TransactionIsolation {
        Default,
        ReadUncommitted,
        ReadCommitted,
        RepeatableRead,
        Serializable,
    }
    struct TransactionSetting {
        scope: VariableScope,
        access: TransactionAccess,
        isolation: TransactionIsolation,
    }
    enum CacheKind {
        Plan,
        Procedure,
        PreparedStatement,
        Library,
    }
    enum CacheFilter {
        All,
        SqlId,
        SchemaId,
    }
    enum SqlReferenceKind {
        Column,
        Relation,
        Function,
        Projection,
        Into,
    }
    struct SqlReference<'a> {
        kind: SqlReferenceKind,
        node: &'a Node,
    }
    struct SqlSymbol<'a> {
        node: &'a Node,
        name: String,
    }
    struct SqlSymbolBinding<'a> {
        node: &'a Node,
        parameter: i64,
    }
    struct EmbeddedSql<'a> {
        sql: String,
        bindings: Vec<SqlReference<'a>>,
    }
    enum ParameterMode { Text, Prepare, Execute }
    #[derive(Clone, Copy)]
    struct ParameterOptions {
        mode: ParameterMode,
        outline: bool,
        from_pl: bool,
        no_unsigned_subtraction: bool,
        projection_names: bool,
        projection_name_limit: usize,
        parameter_base: usize,
    }

    enum ParameterAction { Normal, Preserve, UnaryNegation, Subtraction }
    enum ParameterError { None, Unsupported, InvalidOutline, InvalidArgument }
    enum ParameterOrigin { Literal, UnaryNegation, Subtraction }
    enum ParameterLoweringKind { FoldUnaryExpression, NegateLiteral, AddNegatedOperand }
    enum ParameterScalePolicy { Check, IgnoreNonDecimalScale }
    enum ParameterRepresentation { Ordinary, Extended }
    enum ParameterTypeCheck { Check, SkipType, SkipExtendedType }
    enum NumericSignMode { Source, CompactNegative, PositiveOperand }
    struct NumericSignResult {
        valid: bool,
        integer: u64,
        text_offset: usize,
        compact_text: Vec<u8>,
    }

    struct ParameterToken<'a> {
        kind: LiteralKind,
        integer: u64,
        start: usize,
        end: usize,
        sql_offset: usize,
        text: &'a [u8],
        source: &'a [u8],
        concatenated: bool,
        first_part_length: usize,
        action: ParameterAction,
    }
    struct ParameterValue {
        kind: LiteralKind,
        integer: u64,
        text: Vec<u8>,
        charset: String,
        date_unit: String,
        concatenated: bool,
        first_part_length: usize,
    }
    struct ParameterTokenPolicy {
        action: ParameterAction,
        charset: String,
        positive: bool,
        arithmetic: bool,
        placeholder: i64,
        excluded_placeholder: bool,
    }
    struct ParameterBindingRecord<'a> {
        node: &'a Node,
        raw_index: usize,
        parameter_index: usize,
        origin: ParameterOrigin,
        value: ParameterValue,
        check_type: bool,
        scale_policy: ParameterScalePolicy,
        boolean: bool,
        fixed: bool,
        append: bool,
    }
    struct ParameterLowering<'a> {
        node: &'a Node,
        operation: ParameterLoweringKind,
        value: ParameterValue,
    }
    struct ParameterProjection<'a> {
        node: &'a Node,
        name: Vec<u8>,
        questions: Vec<usize>,
        parameters: Vec<usize>,
        negative_parameters: Vec<usize>,
        escaped_string: bool,
        null_literal: bool,
    }
    struct Parameterization<'a> {
        tokens: Vec<ParameterTokenPolicy>,
        bindings: Vec<ParameterBindingRecord<'a>>,
        lowerings: Vec<ParameterLowering<'a>>,
        projections: Vec<ParameterProjection<'a>>,
        user_variables: Vec<String>,
        unchecked_placeholders: Vec<usize>,
        ps_eligible: bool,
        error: ParameterError,
    }
    struct ParameterSql {
        sql: Vec<u8>,
        folded_in_list: bool,
        error: ParameterError,
    }
    struct ParameterClassification {
        actions: Vec<ParameterAction>,
        error: ParameterError,
    }

    enum StatementClass { Other, Select, Show, Insert, Replace, Update, Delete, Values, Explain }
    enum PlPrefix { None, Anonymous, Routine, Function, Call }
    struct StatementInfo {
        kind: StatementClass,
        pl: PlPrefix,
        explain_offset: usize,
        supports_values_table: bool,
        trace_start: usize,
        trace_end: usize,
    }
    struct RestoreParameter<'a> {
        sql_offset: usize,
        removed_before: usize,
        source: &'a [u8],
    }
    struct RestoredParameter {
        sql_offset: usize,
        source_offset: usize,
    }
    struct RestoredSql {
        sql: Vec<u8>,
        parameters: Vec<RestoredParameter>,
        error: ParameterError,
    }

    struct BatchCompression {
        eligible: bool,
        recognized_rows: bool,
        sql: String,
        rows: usize,
        row_parameters: usize,
        suffix_parameters: usize,
        removed_length: usize,
        source_end: usize,
        output_end: usize,
    }

    enum TableSelectMode {
        None,
        Select,
        Ignore,
        Replace,
    }
    enum ColumnGeneration {
        None,
        Virtual,
        Stored,
    }
    enum ColumnPosition {
        None,
        First,
        Before,
        After,
    }
    enum IndexKind {
        Normal,
        Unique,
        Spatial,
        Fulltext,
        Vector,
    }
    enum IndexMethod {
        Default,
        Hash,
    }
    enum PartitionKind {
        Hash,
        Key,
        Range,
        RangeColumns,
        List,
        ListColumns,
    }
    enum PartitionValues {
        None,
        Range,
        List,
        Default,
    }
    enum ReferenceEvent {
        Update,
        Delete,
    }
    enum ReferenceAction {
        Restrict,
        Cascade,
        SetNull,
        NoAction,
        SetDefault,
    }
    enum DropBehavior {
        Default,
        Restrict,
        Cascade,
    }
    enum TableOrganization {
        Index,
        Heap,
    }
    enum TableRowFormat {
        Redundant,
        Compact,
        Dynamic,
        Compressed,
        Condensed,
    }
    struct TableDefinition<'a> {
        relation: &'a Node,
        elements: &'a [Node],
        options: &'a [Node],
        hints: &'a [Node],
        temporary: bool,
        if_not_exists: bool,
        select_mode: TableSelectMode,
        has_partition: bool,
    }
    struct ColumnDefinition<'a> {
        name: &'a Node,
        data_type: &'a Node,
        attributes: &'a [Node],
        generation: ColumnGeneration,
        position: ColumnPosition,
    }
    struct IndexDefinition<'a> {
        columns: &'a [Node],
        options: &'a [Node],
        kind: IndexKind,
        method: IndexMethod,
        has_name: bool,
        has_partition: bool,
        if_not_exists: bool,
    }
    struct PartitionDefinition<'a> {
        kind: PartitionKind,
        expression: &'a Node,
        elements: &'a [Node],
        has_count: bool,
        count: u64,
        has_subpartition: bool,
        is_template: bool,
    }
    struct PartitionElement<'a> {
        name: &'a Node,
        values: &'a [Node],
        value_kind: PartitionValues,
        subpartitions: &'a [Node],
    }
    struct ForeignKeyDefinition<'a> {
        columns: &'a [Node],
        parent: &'a Node,
        parent_columns: &'a [Node],
        actions: &'a [Node],
        has_name: bool,
        has_index_name: bool,
    }
    struct ReferentialAction {
        event: ReferenceEvent,
        action: ReferenceAction,
    }
    enum ShowFilter {
        None,
        Like,
        Where,
    }
    struct LiteralReference<'a> {
        node: &'a Node,
    }

    enum StatisticsScope {
        All,
        Indexed,
        Hidden,
        Columns,
    }
    enum StatisticsSizeKind {
        Omitted,
        Auto,
        Repeat,
        SkewOnly,
        Count,
    }
    enum StatisticsColumnKind {
        Default,
        Named,
        Extension,
    }

    extern "Rust" {
        fn classify_statement(sql: &[u8], mode: SqlMode) -> StatementInfo;
        fn normalized_sql_length(sql: &[u8]) -> usize;
        fn is_single_statement(sql: &[u8]) -> bool;
        fn restore_parameter_sql(sql: &[u8], parameters: &[RestoreParameter], terminate: bool) -> RestoredSql;
        fn compress_insert(sql: &[u8], values_offset: usize) -> BatchCompression;
        fn compress_values(sql: &[u8], values_offset: usize) -> BatchCompression;
        fn fast_parameterization_allowed(sql: &[u8], mode: SqlMode) -> bool;

        unsafe fn parameterize<'a>(tree: &'a Node, syntax_sql: &[u8], original_sql: &[u8], tokens: &[ParameterToken], options: ParameterOptions) -> Parameterization<'a>;
        fn classify_parameter_tokens(tokens: &[ParameterToken], expected: usize, check_order: bool) -> ParameterClassification;
        fn rewrite_parameter_sql(sql: &[u8], tokens: &[ParameterToken], fold_in: bool) -> ParameterSql;
        fn fold_parameter_in_lists(sql: &[u8]) -> ParameterSql;
        fn unchecked_parameter_types(offsets: &[usize], representations: &[ParameterRepresentation]) -> Vec<ParameterTypeCheck>;

        fn normalize_numeric_sign(kind: LiteralKind, integer: u64, text: &[u8], mode: NumericSignMode) -> NumericSignResult;
        fn integer_literal_is_nonnegative(value: i64, text: &[u8]) -> bool;
        fn identifier_requires_quotes(input: &[u8]) -> bool;
        fn decode_binary_literal(text: &str, notation: BinaryNotation) -> Vec<u8>;
        unsafe fn syntax_literals<'a>(syntax: &'a Node) -> Vec<LiteralReference<'a>>;
        fn index_column_has_direction(node: &Node) -> bool;
        fn primary_key_has_comment(node: &Node) -> bool;
        fn primary_key_comment(node: &Node) -> &Node;
        fn index_hints(node: &Node) -> &[Node];
        fn alter_table_options(node: &Node) -> &[Node];
        fn alter_index_name(node: &Node) -> &Node;
        fn alter_index_new_name(node: &Node) -> &Node;
        fn statistics_clauses(node: &Node) -> &[Node];
        fn statistics_scope(node: &Node) -> StatisticsScope;
        fn statistics_columns(node: &Node) -> &[Node];
        fn statistics_size(node: &Node) -> StatisticsSizeKind;
        fn statistics_bucket_count(node: &Node) -> &Node;
        fn statistics_column_kind(node: &Node) -> StatisticsColumnKind;
        fn statistics_column_name(node: &Node) -> &str;
        fn optimize_relations(node: &Node) -> &[Node];
        fn analyze_relations(node: &Node) -> &[Node];
        fn partition_element_values(node: &Node) -> &Node;
        fn purge_relation(node: &Node) -> &Node;
        fn query_has_mixed_string_array(node: &Node) -> bool;
        fn vector_parameters(option: &Node) -> &[Node];
        fn routine_parameter_list(node: &Node) -> &Node;
        fn trigger_source(node: &Node) -> &Node;
        fn trigger_has_definer(node: &Node) -> bool;
        fn trigger_definer(node: &Node) -> &Node;
        fn trigger_name(node: &Node) -> &Node;
        fn trigger_definition(node: &Node) -> &Node;
        fn trigger_table(node: &Node) -> &Node;
        fn trigger_body(node: &Node) -> &Node;
        fn trigger_after(node: &Node) -> bool;
        fn trigger_event(node: &Node) -> NodeKind;
        fn trigger_has_order(node: &Node) -> bool;
        fn trigger_order_name(node: &Node) -> &Node;
        fn trigger_order_follows(node: &Node) -> bool;
        fn trigger_if_not_exists(node: &Node) -> bool;
        fn trigger_if_exists(node: &Node) -> bool;
        fn routine_name(node: &Node) -> &Node;
        fn routine_has_definer(node: &Node) -> bool;
        fn routine_definer(node: &Node) -> &Node;
        fn routine_body(node: &Node) -> &Node;
        fn routine_parameters(node: &Node) -> &[Node];
        fn routine_return_type(node: &Node) -> &Node;
        fn routine_if_not_exists(node: &Node) -> bool;
        fn routine_if_exists(node: &Node) -> bool;
        fn package_definition(node: &Node) -> &Node;
        fn package_name(node: &Node) -> &Node;
        fn package_has_end_name(node: &Node) -> bool;
        fn package_end_name(node: &Node) -> &Node;
        fn package_drop_body(node: &Node) -> bool;
        fn package_replace(node: &Node) -> bool;
        fn show_full(statement: &Node) -> bool;
        fn show_extended(statement: &Node) -> bool;
        fn show_global(statement: &Node) -> bool;
        fn show_if_not_exists(statement: &Node) -> bool;
        fn show_has_database(statement: &Node) -> bool;
        fn show_database(statement: &Node) -> &Node;
        fn show_object(statement: &Node) -> &Node;
        fn show_filter(statement: &Node) -> ShowFilter;
        fn show_filter_expression(statement: &Node) -> &Node;
        fn show_like_pattern(statement: &Node) -> &Node;
        fn show_like_escape(statement: &Node) -> &Node;
        fn show_has_account(statement: &Node) -> bool;
        fn show_account_name(statement: &Node) -> &str;
        fn show_account_has_host(statement: &Node) -> bool;
        fn show_account_host(statement: &Node) -> &str;
        fn show_has_limit(statement: &Node) -> bool;
        fn show_limit_offset(statement: &Node) -> u64;
        fn show_limit_count(statement: &Node) -> u64;
        unsafe fn table_definition<'a>(node: &'a Node) -> TableDefinition<'a>;
        fn table_definition_query(node: &Node) -> &Node;
        fn table_definition_partition(node: &Node) -> &Node;
        fn table_like_relation(node: &Node) -> &Node;
        fn table_like_source(node: &Node) -> &Node;
        fn table_like_temporary(node: &Node) -> bool;
        fn table_like_if_not_exists(node: &Node) -> bool;
        unsafe fn column_definition<'a>(node: &'a Node) -> ColumnDefinition<'a>;
        fn column_generation_expression(node: &Node) -> &Node;
        fn column_position_name(node: &Node) -> &Node;
        fn column_attribute_expression(attribute: &Node) -> &Node;
        fn column_attribute_text(attribute: &Node) -> &str;
        fn column_srid(attribute: &Node) -> u64;
        unsafe fn index_definition<'a>(node: &'a Node) -> IndexDefinition<'a>;
        fn index_definition_name(node: &Node) -> &Node;
        fn index_definition_table(node: &Node) -> &Node;
        fn index_definition_partition(node: &Node) -> &Node;
        fn index_column_expression(node: &Node) -> &Node;
        fn index_column_has_prefix(node: &Node) -> bool;
        fn index_column_prefix(node: &Node) -> u64;
        fn index_column_descending(node: &Node) -> bool;
        fn primary_key_columns(node: &Node) -> &[Node];
        fn primary_key_method(node: &Node) -> IndexMethod;
        fn primary_key_options(node: &Node) -> &[Node];
        unsafe fn partition_definition<'a>(node: &'a Node) -> PartitionDefinition<'a>;
        fn partition_subpartition(node: &Node) -> &Node;
        unsafe fn partition_element<'a>(node: &'a Node) -> PartitionElement<'a>;
        fn check_constraint_has_name(node: &Node) -> bool;
        fn check_constraint_name(node: &Node) -> &Node;
        fn check_constraint_expression(node: &Node) -> &Node;
        fn check_constraint_enforced(node: &Node) -> bool;
        unsafe fn foreign_key_definition<'a>(node: &'a Node) -> ForeignKeyDefinition<'a>;
        fn foreign_key_name(node: &Node) -> &Node;
        fn foreign_key_index_name(node: &Node) -> &Node;
        fn referential_action(node: &Node) -> ReferentialAction;
        fn table_option_text(node: &Node) -> &str;
        fn table_option_number(node: &Node) -> u64;
        fn table_option_names(node: &Node) -> &[Node];
        fn table_option_properties(node: &Node) -> &[Node];
        fn table_option_charset(node: &Node) -> &Node;
        fn table_option_organization(node: &Node) -> TableOrganization;
        fn table_option_row_format(node: &Node) -> TableRowFormat;
        fn index_option_global(node: &Node) -> bool;
        fn database_name(node: &Node) -> &Node;
        fn database_options(node: &Node) -> &[Node];
        fn database_if_not_exists(node: &Node) -> bool;
        fn drop_if_exists(node: &Node) -> bool;
        fn drop_temporary(node: &Node) -> bool;
        fn drop_relations(node: &Node) -> &[Node];
        fn fork_source(node: &Node) -> &Node;
        fn fork_destination(node: &Node) -> &Node;
        fn rename_actions(node: &Node) -> &[Node];
        fn rename_source(node: &Node) -> &Node;
        fn rename_destination(node: &Node) -> &Node;
        fn truncate_relation(node: &Node) -> &Node;
        fn drop_index_name(node: &Node) -> &Node;
        fn drop_index_relation(node: &Node) -> &Node;
        fn alter_table_relation(node: &Node) -> &Node;
        fn alter_table_actions(node: &Node) -> &[Node];
        fn alter_column_definitions(node: &Node) -> &[Node];
        fn alter_column_name(node: &Node) -> &Node;
        fn alter_column_new_name(node: &Node) -> &Node;
        fn alter_column_has_default(node: &Node) -> bool;
        fn alter_column_default(node: &Node) -> &Node;
        fn alter_column_drop_behavior(node: &Node) -> DropBehavior;
        fn alter_constraint_name(node: &Node) -> &Node;
        fn alter_partition_names(node: &Node) -> &[Node];
        fn alter_partition_definition(node: &Node) -> &Node;
        fn alter_partition_additions(node: &Node) -> &[Node];
        fn alter_partition_exchange_name(node: &Node) -> &Node;
        fn alter_partition_exchange_table(node: &Node) -> &Node;
        fn alter_table_renamed_relation(node: &Node) -> &Node;
        fn alter_charset_name(node: &Node) -> &Node;
        fn alter_charset_has_collation(node: &Node) -> bool;
        fn alter_charset_collation(node: &Node) -> &Node;
        unsafe fn embedded_sql_references<'a>(statement: &'a Node) -> Vec<SqlReference<'a>>;
        unsafe fn embedded_sql_symbols<'a>(statement: &'a Node, sql: &str) -> Vec<SqlSymbol<'a>>;
        unsafe fn rewrite_embedded_sql<'a>(
            statement: &'a Node,
            sql: &str,
            symbols: &[SqlSymbolBinding],
        ) -> EmbeddedSql<'a>;
        fn transaction_setting(statement: &Node) -> TransactionSetting;
        fn empty_query_has_comment(statement: &Node) -> bool;
        fn freeze_is_major(statement: &Node) -> bool;
        fn cache_flush_kind(statement: &Node) -> CacheKind;
        fn cache_flush_has_namespace(statement: &Node) -> bool;
        fn cache_flush_namespace(statement: &Node) -> &str;
        fn cache_flush_filter(statement: &Node) -> CacheFilter;
        fn cache_flush_filter_value(statement: &Node) -> &Node;
        fn cache_flush_has_databases(statement: &Node) -> bool;
        fn cache_flush_databases(statement: &Node) -> &str;
        fn kv_cache_has_name(statement: &Node) -> bool;
        fn kv_cache_name(statement: &Node) -> &str;
        fn configuration_assignments(statement: &Node) -> &[Node];
        fn configuration_name(assignment: &Node) -> &str;
        fn configuration_value(assignment: &Node) -> &Node;
        fn configuration_comment(assignment: &Node) -> &str;
        fn tracepoint_options(statement: &Node) -> &[Node];
        fn tracepoint_value(option: &Node) -> &Node;
        fn routine_characteristics(node: &Node) -> &[Node];
        fn routine_invoker_rights(node: &Node) -> bool;
        fn routine_data_access(node: &Node) -> RoutineDataAccess;
        unsafe fn view_definition<'a>(node: &'a Node) -> ViewDefinition<'a>;
        fn equivalent_expressions(left: &Node, right: &Node) -> bool;
        fn block_declarations(node: &Node) -> &[Node];
        fn block_statements(node: &Node) -> &[Node];
        fn labeled_body(node: &Node) -> &Node;
        fn block_label(node: &Node) -> &Node;
        fn block_has_end_label(node: &Node) -> bool;
        fn block_end_label(node: &Node) -> &Node;
        fn declaration_names(node: &Node) -> &[Node];
        fn declaration_type(node: &Node) -> &Node;
        fn declaration_has_default(node: &Node) -> bool;
        fn declaration_default(node: &Node) -> &Node;
        fn control_condition(node: &Node) -> &Node;
        fn control_body(node: &Node) -> &Node;
        fn control_has_alternative(node: &Node) -> bool;
        fn control_alternative(node: &Node) -> &Node;
        fn case_branches(node: &Node) -> &[Node];
        fn return_expression(node: &Node) -> &Node;
        fn loop_target(node: &Node) -> &Node;
        fn cursor_name(node: &Node) -> &Node;
        fn cursor_query(node: &Node) -> &Node;
        fn cursor_fetch_targets(node: &Node) -> &[Node];
        fn handler_conditions(node: &Node) -> &[Node];
        fn handler_body(node: &Node) -> &Node;
        fn handler_continues(node: &Node) -> bool;
        fn condition_name(node: &Node) -> &Node;
        fn condition_declaration_value(node: &Node) -> &Node;
        fn condition_is_sqlstate(node: &Node) -> bool;
        fn condition_sqlstate(node: &Node) -> &str;
        fn condition_error_code(node: &Node) -> u64;
        fn signal_has_condition(node: &Node) -> bool;
        fn signal_condition(node: &Node) -> &Node;
        fn signal_items(node: &Node) -> &[Node];
        fn signal_item_name(node: &Node) -> &str;
        fn signal_item_expression(node: &Node) -> &Node;
        fn routine_call_names(node: &Node) -> &[Node];
        fn routine_call_arguments(node: &Node) -> &[Node];
        fn routine_sql_statement(node: &Node) -> &Node;
        fn package_declarations(node: &Node) -> &[Node];
        fn package_has_initialization(node: &Node) -> bool;
        fn package_initialization(node: &Node) -> &Node;
        fn subprogram_declaration(node: &Node) -> &Node;
        fn subprogram_definition(node: &Node) -> &Node;
        fn subprogram_name(node: &Node) -> &Node;
        fn subprogram_parameters(node: &Node) -> &[Node];
        fn subprogram_return_type(node: &Node) -> &Node;
        fn routine_parameter_name(node: &Node) -> &Node;
        fn routine_parameter_type(node: &Node) -> &Node;
        fn routine_parameter_input(node: &Node) -> bool;
        fn routine_parameter_output(node: &Node) -> bool;
        fn routine_parameter_has_default(node: &Node) -> bool;
        fn routine_parameter_default(node: &Node) -> &Node;
        fn interface_name(node: &Node) -> &Node;
        fn row_type_names(node: &Node) -> &[Node];

        fn setting_assignments(statement: &Node) -> &[Node];
        fn setting_scope(assignment: &Node) -> VariableScope;
        fn setting_charset(assignment: &Node) -> &Node;
        fn setting_has_collation(assignment: &Node) -> bool;
        fn setting_collation(assignment: &Node) -> &Node;
        fn kill_query_only(statement: &Node) -> bool;
        fn kill_target(statement: &Node) -> &Node;
        fn diagnostics_stacked(statement: &Node) -> bool;
        fn diagnostics_has_condition(statement: &Node) -> bool;
        fn diagnostics_condition(statement: &Node) -> &Node;
        fn diagnostics_items(statement: &Node) -> &[Node];
        fn diagnostics_target(item: &Node) -> &Node;
        fn diagnostics_item_name(item: &Node) -> &str;

        fn explain_format(explain: &Node) -> ExplainFormat;
        fn explain_display(explain: &Node) -> ExplainDisplay;
        fn explain_has_into_table(explain: &Node) -> bool;
        fn explain_into_table(explain: &Node) -> &Node;
        fn explain_has_statement_id(explain: &Node) -> bool;
        fn explain_statement_id(explain: &Node) -> &Node;

        fn is_call_expression(expression: &Node) -> bool;
        fn explain_statement(explain: &Node) -> &Node;
        fn named_argument_name(argument: &Node) -> &Node;
        fn named_argument_value(argument: &Node) -> &Node;
        fn identifier_is_quoted(identifier: &Node) -> bool;

        type RelationReferences<'a>;
        unsafe fn relation_references<'a>(query: &'a Node) -> Box<RelationReferences<'a>>;
        fn advance_relation(self: &mut RelationReferences<'_>) -> bool;
        unsafe fn relation_reference<'a>(self: &RelationReferences<'a>) -> &'a Node;
        fn collation_argument(expression: &Node) -> &Node;
        fn collation_specification(expression: &Node) -> &Node;
        fn collation_name(expression: &Node) -> &str;
        type Scanner<'a>;
        // The SQL buffer must remain valid and unchanged until the scanner is dropped.
        unsafe fn begin_scan<'a>(sql: &'a [u8], options: ScanOptions) -> Box<Scanner<'a>>;
        fn advance(self: &mut Scanner<'_>) -> bool;
        fn finish_scan(scanner: Box<Scanner<'_>>) -> ScanResult;
        fn split_sql(sql: &[u8], mode: SqlMode, first_only: bool) -> StatementSplits;
        fn parse_search_query(input: &[u8]) -> SearchTree;
        fn inspect_expression(sql: &[u8], mode: SqlMode) -> ExpressionInfo;
        fn transaction_options(node: &Node) -> TransactionOptions;
        fn prepared_source(node: &Node) -> &Node;
        fn execution_arguments(node: &Node) -> &[Node];
        fn access_options(statement: &Node) -> AccessOptions;
        fn access_accounts(statement: &Node) -> &[Node];
        fn access_roles(statement: &Node) -> &[Node];
        fn access_privileges(statement: &Node) -> &[Node];
        fn access_role_selection(statement: &Node) -> RoleSelection;
        fn account_kind(spec: &Node) -> AccountKind;
        unsafe fn account_identity<'a>(principal: &'a Node) -> AccountIdentity<'a>;
        fn account_name_form(spec: &Node) -> AccountNameForm;
        fn account_name(spec: &Node) -> &str;
        fn account_has_host(spec: &Node) -> bool;
        fn account_host(spec: &Node) -> &str;
        fn account_authentication(spec: &Node) -> AuthenticationKind;
        fn account_password(spec: &Node) -> &str;
        fn account_has_plugin(spec: &Node) -> bool;
        fn account_plugin(spec: &Node) -> &str;
        fn privilege_name(privilege: &Node) -> &str;
        fn privilege_columns(privilege: &Node) -> &[Node];
        fn access_target(statement: &Node) -> PrivilegeTarget;
        fn access_target_database(statement: &Node) -> &str;
        fn access_target_object(statement: &Node) -> &str;
        fn query_options(query: &Node) -> &[Node];
        fn query_projection(query: &Node) -> &[Node];
        fn query_sources(query: &Node) -> &[Node];
        fn query_has_filter(query: &Node) -> bool;
        fn query_filter(query: &Node) -> &Node;
        fn query_has_grouping(query: &Node) -> bool;
        fn query_grouping(query: &Node) -> &Node;
        fn query_has_having(query: &Node) -> bool;
        fn query_having(query: &Node) -> &Node;
        fn query_windows(query: &Node) -> &[Node];
        fn query_order(query: &Node) -> &[Node];
        fn query_has_limit(query: &Node) -> bool;
        fn query_limit(query: &Node) -> &Node;
        fn query_has_lock(query: &Node) -> bool;
        fn query_lock(query: &Node) -> &Node;
        fn query_hints(query: &Node) -> &[Node];
        fn query_ctes(query: &Node) -> &[Node];
        fn query_recursive(query: &Node) -> bool;
        fn query_set_operator(query: &Node) -> SetOperator;
        fn query_set_operands(query: &Node) -> &[Node];
        fn query_into_position(query: &Node) -> IntoPosition;
        fn query_into_variables(query: &Node) -> &[Node];
        fn query_approximate(query: &Node) -> bool;
        fn query_vector_parameters(query: &Node) -> &[Node];
        fn vector_parameter_name(parameter: &Node) -> &str;
        fn vector_parameter_value(parameter: &Node) -> &Node;
        fn query_ignore(query: &Node) -> bool;
        fn query_assignments(query: &Node) -> &[Node];
        fn delete_targets(query: &Node) -> &[Node];
        fn projection_expression(projection: &Node) -> &Node;
        fn projection_alias_form(projection: &Node) -> AliasForm;
        fn projection_alias(projection: &Node) -> &Node;
        fn grouping_keys(grouping: &Node) -> &[Node];
        fn grouping_with_rollup(grouping: &Node) -> bool;
        fn sort_expression(sort: &Node) -> &Node;
        fn sort_direction(sort: &Node) -> SortDirection;
        fn limit_count(limit: &Node) -> &Node;
        fn limit_has_offset(limit: &Node) -> bool;
        fn limit_offset(limit: &Node) -> &Node;
        fn limit_comma_form(limit: &Node) -> bool;
        fn lock_kind(lock: &Node) -> LockKind;
        fn lock_wait(lock: &Node) -> LockWait;
        fn lock_wait_seconds(lock: &Node) -> &str;
        fn cte_name(cte: &Node) -> &Node;
        fn cte_columns(cte: &Node) -> &[Node];
        fn cte_query(cte: &Node) -> &Node;
        fn assignment_target(assignment: &Node) -> &Node;
        fn assignment_value(assignment: &Node) -> &Node;
        fn insert_target(insert: &Node) -> &Node;
        fn insert_columns(insert: &Node) -> &[Node];
        fn insert_source(insert: &Node) -> InsertSource;
        fn insert_query(insert: &Node) -> &Node;
        fn insert_rows(insert: &Node) -> &[Node];
        fn insert_assignments(insert: &Node) -> &[Node];
        fn insert_on_duplicate(insert: &Node) -> &[Node];
        fn insert_replace(insert: &Node) -> bool;
        fn row_elements(row: &Node) -> &[Node];
        fn name_parts(name: &Node) -> &[Node];
        fn name_is_wildcard(name: &Node) -> bool;
        fn relation_has_database(relation: &Node) -> bool;
        fn relation_database(relation: &Node) -> &Node;
        fn relation_name(relation: &Node) -> &Node;
        fn table_source(table: &Node) -> &Node;
        fn table_has_alias(table: &Node) -> bool;
        fn table_alias(table: &Node) -> &Node;
        fn table_columns(table: &Node) -> &[Node];
        fn table_lateral(table: &Node) -> bool;
        fn table_parenthesized(table: &Node) -> bool;
        fn table_partitions(table: &Node) -> &[Node];
        fn table_index_hints(table: &Node) -> &[Node];
        fn table_has_sample(table: &Node) -> bool;
        fn table_sample(table: &Node) -> &Node;
        fn table_has_snapshot(table: &Node) -> bool;
        fn table_snapshot(table: &Node) -> &Node;
        fn sample_method(sample: &Node) -> SampleMethod;
        fn sample_scope(sample: &Node) -> SampleScope;
        fn sample_percent(sample: &Node) -> &Node;
        fn sample_has_seed(sample: &Node) -> bool;
        fn sample_seed(sample: &Node) -> &Node;
        fn index_hint_kind(hint: &Node) -> IndexHintKind;
        fn index_hint_names(hint: &Node) -> &[Node];
        fn join_kind(join: &Node) -> JoinKind;
        fn join_left(join: &Node) -> &Node;
        fn join_right(join: &Node) -> &Node;
        fn join_condition(join: &Node) -> JoinCondition;
        fn join_predicate(join: &Node) -> &Node;
        fn join_columns(join: &Node) -> &[Node];
        fn values_rows(values: &Node) -> &[Node];
        fn collection_table_expression(table: &Node) -> &Node;
        fn is_literal(expression: &Node) -> bool;
        unsafe fn expression_literal<'a>(expression: &'a Node) -> ExpressionLiteral<'a>;
        fn integer_literal_value(literal: &Node) -> u64;
        fn placeholder_index(parameter: &Node) -> usize;
        fn placeholder_name(parameter: &Node) -> &str;
        fn parenthesized_expression(expression: &Node) -> &Node;
        fn unary_operator(expression: &Node) -> UnaryOperator;
        fn unary_operand(expression: &Node) -> &Node;
        fn binary_operator(expression: &Node) -> BinaryOperator;
        fn binary_left(expression: &Node) -> &Node;
        fn binary_right(expression: &Node) -> &Node;
        fn like_has_escape(expression: &Node) -> bool;
        fn like_escape(expression: &Node) -> &Node;
        fn predicate_negated(expression: &Node) -> bool;
        fn predicate_subject(expression: &Node) -> &Node;
        fn between_lower(expression: &Node) -> &Node;
        fn between_upper(expression: &Node) -> &Node;
        fn is_test(expression: &Node) -> &Node;
        fn membership_source(expression: &Node) -> MembershipSource;
        fn membership_expressions(expression: &Node) -> &[Node];
        fn membership_query(expression: &Node) -> &Node;
        fn quantified_comparison(expression: &Node) -> ComparisonQuantifier;
        fn quantified_query(expression: &Node) -> &Node;
        fn exists_query(expression: &Node) -> &Node;
        fn call_name(call: &Node) -> &str;
        fn call_qualifiers(call: &Node) -> &[Node];
        fn call_arguments(call: &Node) -> &[Node];
        fn aggregate_quantifier(aggregate: &Node) -> AggregateQuantifier;
        fn aggregate_order(aggregate: &Node) -> &[Node];
        fn aggregate_has_separator(aggregate: &Node) -> bool;
        fn aggregate_separator(aggregate: &Node) -> &Node;
        fn case_has_operand(expression: &Node) -> bool;
        fn case_operand(expression: &Node) -> &Node;
        fn case_arms(expression: &Node) -> &[Node];
        fn case_has_else(expression: &Node) -> bool;
        fn case_else(expression: &Node) -> &Node;
        fn case_condition(arm: &Node) -> &Node;
        fn case_result(arm: &Node) -> &Node;
        fn cast_argument(cast: &Node) -> &Node;
        fn cast_target(cast: &Node) -> &Node;
        fn cast_ignore(cast: &Node) -> bool;
        fn type_name(target: &Node) -> &str;
        fn type_parameters(target: &Node) -> &[Node];
        fn type_charset(target: &Node) -> &str;
        fn type_collation(target: &Node) -> &str;
        fn type_unsigned(target: &Node) -> bool;
        fn type_zerofill(target: &Node) -> bool;
        unsafe fn type_syntax<'a>(target: &'a Node) -> TypeSyntax<'a>;
        fn type_element(target: &Node) -> &Node;
        fn type_key(target: &Node) -> &Node;
        fn type_value(target: &Node) -> &Node;
        fn type_members(target: &Node) -> &[Node];
        fn variable_name(expression: &Node) -> &str;
        fn variable_scope(expression: &Node) -> VariableScope;
        fn variable_assignment_value(expression: &Node) -> &Node;
        fn window_function(expression: &Node) -> &Node;
        fn window_definition(expression: &Node) -> &Node;
        fn window_has_reference(window: &Node) -> bool;
        fn window_reference(window: &Node) -> &Node;
        fn window_partition(window: &Node) -> &[Node];
        fn window_order(window: &Node) -> &[Node];
        fn window_has_frame(window: &Node) -> bool;
        fn window_frame(window: &Node) -> &Node;
        fn named_window_name(window: &Node) -> &Node;
        fn named_window_definition(window: &Node) -> &Node;
        fn window_null_treatment(function: &Node) -> NullTreatment;
        fn window_nth_origin(function: &Node) -> NthOrigin;
        fn frame_unit(frame: &Node) -> FrameUnit;
        fn frame_start(frame: &Node) -> &Node;
        fn frame_has_end(frame: &Node) -> bool;
        fn frame_end(frame: &Node) -> &Node;
        fn frame_bound(bound: &Node) -> FrameBound;
        fn frame_bound_offset(bound: &Node) -> &Node;
        fn lambda_parameters(lambda: &Node) -> &[Node];
        fn lambda_body(lambda: &Node) -> &Node;
        fn collection_lambda(call: &Node) -> &Node;
        fn vector_metric(call: &Node) -> VectorMetric;
        fn fulltext_columns(expression: &Node) -> &[Node];
        fn fulltext_search(expression: &Node) -> &Node;
        fn fulltext_mode(expression: &Node) -> FulltextMode;
        fn json_document(expression: &Node) -> &Node;
        fn json_path(expression: &Node) -> &Node;
        fn json_returning(expression: &Node) -> &Node;
        fn json_options(expression: &Node) -> JsonOptions;
        fn json_error_clause(expression: &Node) -> &Node;
        fn json_empty_clause(expression: &Node) -> &Node;
        fn json_mismatch_clause(expression: &Node) -> &Node;
        fn json_error_default(expression: &Node) -> &Node;
        fn json_empty_default(expression: &Node) -> &Node;
        fn json_table_columns(table: &Node) -> &[Node];
        fn json_column_kind(column: &Node) -> JsonColumnKind;
        fn json_column_name(column: &Node) -> &Node;
        fn json_column_type(column: &Node) -> &Node;
        fn json_nested_columns(column: &Node) -> &[Node];
        fn hint_has_query_block(hint: &Node) -> bool;
        fn hint_query_block(hint: &Node) -> &Node;
        fn hint_tables(hint: &Node) -> &[Node];
        fn hint_table_relation(table: &Node) -> &Node;
        fn hint_table_has_query_block(table: &Node) -> bool;
        fn hint_table_query_block(table: &Node) -> &Node;
        fn hint_leading(hint: &Node) -> &Node;
        fn hint_leading_group(group: &Node) -> &[Node];
        fn hint_query_block_groups(hint: &Node) -> &[Node];
        fn hint_query_block_names(group: &Node) -> &[Node];
        fn hint_distribution_methods(hint: &Node) -> &[Node];
        fn hint_has_target(hint: &Node) -> bool;
        fn hint_target(hint: &Node) -> &Node;
        fn hint_parallel_policy(hint: &Node) -> ParallelPolicy;
        fn hint_parallel_degree(hint: &Node) -> &Node;
        fn hint_timeout(hint: &Node) -> &Node;
        fn hint_topk_minimum(hint: &Node) -> &Node;
        fn hint_topk_precision(hint: &Node) -> &Node;
        fn hint_sampling_level(hint: &Node) -> &Node;
        fn hint_has_sampling_blocks(hint: &Node) -> bool;
        fn hint_sampling_blocks(hint: &Node) -> &Node;
        fn hint_index_name(hint: &Node) -> &Node;
        fn hint_has_index_prefix(hint: &Node) -> bool;
        fn hint_index_prefix(hint: &Node) -> &Node;
        fn hint_has_condition(hint: &Node) -> bool;
        fn hint_condition(hint: &Node) -> &Node;
        fn hint_merge_direction(hint: &Node) -> MergeDirection;
        fn hint_read_consistency(hint: &Node) -> ReadConsistency;
        fn hint_plan_cache_policy(hint: &Node) -> PlanCachePolicy;
        fn hint_parameter_name(hint: &Node) -> &Node;
        fn hint_parameter_value(hint: &Node) -> &Node;
        fn hint_has_sort_method(hint: &Node) -> bool;
        fn hint_sort_method(hint: &Node) -> &Node;
        fn parse_sql(sql: &[u8], mode: SqlMode, parse_mode: ParseMode) -> ParseTree;
    }
}

use crate::access::{
    access_accounts, access_options, access_privileges, access_role_selection, access_roles,
    access_target, access_target_database, access_target_object, account_authentication,
    account_has_host, account_has_plugin, account_host, account_identity, account_kind,
    account_name, account_name_form, account_password, account_plugin, privilege_columns,
    privilege_name,
};
use crate::fts::parse_search_query;
use crate::inspect::inspect_expression;
use crate::scan::{begin_scan, finish_scan, Scanner};

fn prepared_source(node: &Node) -> &Node {
    &node.children[0]
}

fn execution_arguments(node: &Node) -> &[Node] {
    &node.children
}

fn transaction_options(node: &Node) -> ffi::TransactionOptions {
    let mut options = ffi::TransactionOptions {
        consistent_snapshot: false,
        access: ffi::TransactionAccess::Default,
    };
    for option in &node.children {
        match option.kind {
            Kind::ConsistentSnapshot => options.consistent_snapshot = true,
            Kind::ReadOnly => options.access = ffi::TransactionAccess::ReadOnly,
            Kind::ReadWrite => options.access = ffi::TransactionAccess::ReadWrite,
            _ => {}
        }
    }
    options
}

fn number_parameters(
    node: &mut Node,
    count: &mut usize,
    names: &mut Vec<String>,
) -> Result<(), usize> {
    fn collect<'a>(node: &'a mut Node, parameters: &mut Vec<&'a mut Node>) {
        if node.kind == Kind::Parameter {
            parameters.push(node);
        } else {
            for child in &mut node.children {
                collect(child, parameters);
            }
        }
    }
    let mut parameters = Vec::new();
    collect(node, &mut parameters);
    parameters.sort_by_key(|parameter| parameter.start);
    for node in parameters {
        if node.text.is_empty() {
            if !names.is_empty() {
                return Err(node.start);
            }
            node.value = *count as u64;
            *count += 1;
        } else {
            if *count != names.len() {
                return Err(node.start);
            }
            let index = names
                .iter()
                .position(|name| name.eq_ignore_ascii_case(&node.text));
            node.value = index.unwrap_or_else(|| {
                names.push(node.text.clone());
                *count += 1;
                names.len() - 1
            }) as u64;
        }
    }
    Ok(())
}

pub fn parse_sql(sql: &[u8], mode: SqlMode, parse_mode: ffi::ParseMode) -> ffi::ParseTree {
    let mut result = ffi::ParseTree {
        root: Node::missing(),
        expanded_sql: String::new(),
        parameters: 0,
        parameter_names: Vec::new(),
        named_parameters: false,
        error: String::new(),
        error_offset: 0,
    };
    let sql = &sql[..normalized_sql_length(sql)];
    let sql = match std::str::from_utf8(sql) {
        Ok(sql) => sql,
        Err(error) => {
            result.error_offset = error.valid_up_to();
            result.error = error.to_string();
            return result;
        }
    };
    let source = if parse_mode == ffi::ParseMode::RoutineDefinition {
        let mut source = String::with_capacity("CREATE\n".len() + sql.len());
        source.push_str("CREATE\n");
        source.push_str(sql);
        std::borrow::Cow::Owned(source)
    } else {
        std::borrow::Cow::Borrowed(sql)
    };
    let sql = source.as_ref();
    let parse = |text: &str| match parse_mode {
        ffi::ParseMode::Routine => grammar::sql::routine_body(text, mode),
        ffi::ParseMode::Trigger => grammar::sql::trigger(text, mode),
        ffi::ParseMode::Statistics => grammar::sql::statistics_options(text, mode),
        ffi::ParseMode::Expression => grammar::sql::expression(text, mode),
        ffi::ParseMode::Sql | ffi::ParseMode::RoutineDefinition => grammar::sql::query(text, mode),
        _ => unreachable!("unknown parser entry point"),
    };
    let parsed = grammar::executable_sql(sql, mode, true).and_then(|mut text| {
        let mut parsed = parse(&text);
        if parsed.is_err() && matches!(text, std::borrow::Cow::Owned(_)) {
            text = grammar::executable_sql(sql, mode, false)?;
            parsed = parse(&text);
        }
        if let std::borrow::Cow::Owned(sql) = text {
            result.expanded_sql = sql;
        }
        parsed
    });
    match parsed {
        Ok(query)
            if parse_mode == ffi::ParseMode::RoutineDefinition
                && !matches!(query.kind, Kind::CreateProcedure | Kind::CreateFunction) =>
        {
            result.error_offset = query.start;
            result.error = "expected procedure or function definition".into();
        }
        Ok(mut query) => {
            match number_parameters(
                &mut query,
                &mut result.parameters,
                &mut result.parameter_names,
            ) {
                Ok(()) => {
                    result.named_parameters = result
                        .parameter_names
                        .iter()
                        .any(|name| !name.as_bytes()[1].is_ascii_digit());
                    result.root = query;
                }
                Err(offset) => {
                    result.error_offset = offset;
                    result.error = "ordinal and named parameters cannot be combined".into();
                }
            }
        }
        Err(error) => {
            result.error_offset = if error.location.offset == sql.len() {
                grammar::sql::last_token_start(sql, mode).unwrap()
            } else {
                error.location.offset
            };
            result.error = error.to_string();
        }
    }
    if result.expanded_sql.is_empty() {
        if let std::borrow::Cow::Owned(source) = source {
            result.expanded_sql = source;
        }
    }
    result
}

use crate::query::{
    assignment_target, assignment_value, collection_table_expression, cte_columns, cte_name,
    cte_query, delete_targets, explain_display, explain_format, explain_has_into_table,
    explain_has_statement_id, explain_into_table, explain_statement, explain_statement_id,
    grouping_keys, grouping_with_rollup, hint_condition, hint_distribution_methods,
    hint_has_condition, hint_has_index_prefix, hint_has_query_block, hint_has_sampling_blocks,
    hint_has_sort_method, hint_has_target, hint_index_name, hint_index_prefix, hint_leading,
    hint_leading_group, hint_merge_direction, hint_parallel_degree, hint_parallel_policy,
    hint_parameter_name, hint_parameter_value, hint_plan_cache_policy, hint_query_block,
    hint_query_block_groups, hint_query_block_names, hint_read_consistency, hint_sampling_blocks,
    hint_sampling_level, hint_sort_method, hint_table_has_query_block, hint_table_query_block,
    hint_table_relation, hint_tables, hint_target, hint_timeout, hint_topk_minimum,
    hint_topk_precision, identifier_is_quoted, index_hint_kind, index_hint_names,
    insert_assignments, insert_columns, insert_on_duplicate, insert_query, insert_replace,
    insert_rows, insert_source, insert_target, join_columns, join_condition, join_kind, join_left,
    join_predicate, join_right, limit_comma_form, limit_count, limit_has_offset, limit_offset,
    lock_kind, lock_wait, lock_wait_seconds, name_is_wildcard, name_parts, projection_alias,
    projection_alias_form, projection_expression, query_approximate, query_assignments, query_ctes,
    query_filter, query_grouping, query_has_filter, query_has_grouping, query_has_having,
    query_has_limit, query_has_lock, query_having, query_hints, query_ignore, query_into_position,
    query_into_variables, query_limit, query_lock, query_options, query_order, query_projection,
    query_recursive, query_set_operands, query_set_operator, query_sources,
    query_vector_parameters, query_windows, relation_database, relation_has_database,
    relation_name, relation_references, row_elements, sample_has_seed, sample_method,
    sample_percent, sample_scope, sample_seed, sort_direction, sort_expression, table_alias,
    table_columns, table_has_alias, table_has_sample, table_has_snapshot, table_index_hints,
    table_lateral, table_parenthesized, table_partitions, table_sample, table_snapshot,
    table_source, values_rows, vector_parameter_name, vector_parameter_value, RelationReferences,
};

use crate::expression::{
    aggregate_has_separator, aggregate_order, aggregate_quantifier, aggregate_separator,
    between_lower, between_upper, binary_left, binary_operator, binary_right, call_arguments,
    call_name, call_qualifiers, case_arms, case_condition, case_else, case_has_else,
    case_has_operand, case_operand, case_result, cast_argument, cast_ignore, cast_target,
    collation_argument, collation_name, collation_specification, collection_lambda, exists_query,
    expression_literal, frame_bound, frame_bound_offset, frame_end, frame_has_end, frame_start,
    frame_unit, fulltext_columns, fulltext_mode, fulltext_search, integer_literal_value,
    is_call_expression, is_literal, is_test, json_column_kind, json_column_name, json_column_type,
    json_document, json_empty_clause, json_empty_default, json_error_clause, json_error_default,
    json_mismatch_clause, json_nested_columns, json_options, json_path, json_returning,
    json_table_columns, lambda_body, lambda_parameters, like_escape, like_has_escape,
    membership_expressions, membership_query, membership_source, named_argument_name,
    named_argument_value, named_window_definition, named_window_name, parenthesized_expression,
    placeholder_index, placeholder_name, predicate_negated, predicate_subject,
    quantified_comparison, quantified_query, type_charset, type_collation, type_element, type_key,
    type_members, type_name, type_parameters, type_syntax, type_unsigned, type_value,
    type_zerofill, unary_operand, unary_operator, variable_assignment_value, variable_name,
    variable_scope, vector_metric, window_definition, window_frame, window_function,
    window_has_frame, window_has_reference, window_nth_origin, window_null_treatment, window_order,
    window_partition, window_reference,
};

use crate::command::{
    diagnostics_condition, diagnostics_has_condition, diagnostics_item_name, diagnostics_items,
    diagnostics_stacked, diagnostics_target, kill_query_only, kill_target, setting_assignments,
    setting_charset, setting_collation, setting_has_collation, setting_scope,
};

use crate::routine::{
    block_declarations, block_end_label, block_has_end_label, block_label, block_statements,
    case_branches, condition_declaration_value, condition_error_code, condition_is_sqlstate,
    condition_name, condition_sqlstate, control_alternative, control_body, control_condition,
    control_has_alternative, cursor_fetch_targets, cursor_name, cursor_query, declaration_default,
    declaration_has_default, declaration_names, declaration_type, handler_body, handler_conditions,
    handler_continues, interface_name, labeled_body, loop_target, package_declarations,
    package_has_initialization, package_initialization, return_expression, routine_call_arguments,
    routine_call_names, routine_parameter_default, routine_parameter_has_default,
    routine_parameter_input, routine_parameter_name, routine_parameter_output,
    routine_parameter_type, routine_sql_statement, row_type_names, signal_condition,
    signal_has_condition, signal_item_expression, signal_item_name, signal_items,
    subprogram_declaration, subprogram_definition, subprogram_name, subprogram_parameters,
    subprogram_return_type,
};

use crate::ddl::view_definition;
use crate::expression::equivalent_expressions;

use crate::routine::{routine_characteristics, routine_data_access, routine_invoker_rights};

use crate::command::{
    cache_flush_databases, cache_flush_filter, cache_flush_filter_value, cache_flush_has_databases,
    cache_flush_has_namespace, cache_flush_kind, cache_flush_namespace, configuration_assignments,
    configuration_comment, configuration_name, configuration_value, empty_query_has_comment,
    freeze_is_major, kv_cache_has_name, kv_cache_name, tracepoint_options, tracepoint_value,
    transaction_setting,
};
use crate::inspect::embedded_sql_references;

use crate::ddl::{
    alter_charset_collation, alter_charset_has_collation, alter_charset_name, alter_column_default,
    alter_column_definitions, alter_column_drop_behavior, alter_column_has_default,
    alter_column_name, alter_column_new_name, alter_constraint_name, alter_partition_additions,
    alter_partition_definition, alter_partition_exchange_name, alter_partition_exchange_table,
    alter_partition_names, alter_table_actions, alter_table_relation, alter_table_renamed_relation,
    check_constraint_enforced, check_constraint_expression, check_constraint_has_name,
    check_constraint_name, column_attribute_expression, column_attribute_text, column_definition,
    column_generation_expression, column_position_name, column_srid, database_if_not_exists,
    database_name, database_options, drop_if_exists, drop_index_name, drop_index_relation,
    drop_relations, drop_temporary, foreign_key_definition, foreign_key_index_name,
    foreign_key_name, fork_destination, fork_source, index_column_descending,
    index_column_expression, index_column_has_prefix, index_column_prefix, index_definition,
    index_definition_name, index_definition_partition, index_definition_table, index_option_global,
    partition_definition, partition_element, partition_subpartition, primary_key_columns,
    primary_key_method, primary_key_options, referential_action, rename_actions,
    rename_destination, rename_source, table_definition, table_definition_partition,
    table_definition_query, table_like_if_not_exists, table_like_relation, table_like_source,
    table_like_temporary, table_option_charset, table_option_names, table_option_number,
    table_option_organization, table_option_properties, table_option_row_format, table_option_text,
    truncate_relation,
};

use crate::command::{
    show_account_has_host, show_account_host, show_account_name, show_database, show_extended,
    show_filter, show_filter_expression, show_full, show_global, show_has_account,
    show_has_database, show_has_limit, show_if_not_exists, show_like_escape, show_like_pattern,
    show_limit_count, show_limit_offset, show_object,
};

use crate::routine::{
    package_definition, package_drop_body, package_end_name, package_has_end_name, package_name,
    package_replace, routine_body, routine_definer, routine_has_definer, routine_if_exists,
    routine_if_not_exists, routine_name, routine_parameters, routine_return_type,
};

use crate::routine::{
    trigger_after, trigger_body, trigger_definer, trigger_definition, trigger_event,
    trigger_has_definer, trigger_has_order, trigger_if_exists, trigger_if_not_exists, trigger_name,
    trigger_order_follows, trigger_order_name, trigger_source, trigger_table,
};

use crate::routine::routine_parameter_list;

use crate::query::vector_parameters;

use crate::inspect::query_has_mixed_string_array;

use crate::ddl::{analyze_relations, optimize_relations, purge_relation};
use crate::statistics::{
    statistics_bucket_count, statistics_clauses, statistics_column_kind, statistics_column_name,
    statistics_columns, statistics_scope, statistics_size,
};

use crate::ddl::partition_element_values;

use crate::ddl::{alter_index_name, alter_index_new_name, alter_table_options};
use crate::inspect::syntax_literals;
use crate::lex::{decode_binary_literal, identifier_requires_quotes};
use crate::embedded_sql::{embedded_sql_symbols, rewrite_embedded_sql};

use crate::ddl::{
    index_column_has_direction, index_hints, primary_key_comment, primary_key_has_comment,
};

use crate::parameterize::{parameterize, classify_parameter_tokens, rewrite_parameter_sql, fold_parameter_in_lists, unchecked_parameter_types};

use crate::prefix::{classify_statement, normalized_sql_length, is_single_statement, split_sql, fast_parameterization_allowed};
use crate::batch::{compress_insert, compress_values, restore_parameter_sql};

use crate::lex::{normalize_numeric_sign, integer_literal_is_nonnegative};
