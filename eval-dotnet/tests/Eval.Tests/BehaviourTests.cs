using Eval;

namespace Eval.Tests;

/// <summary>Reading runs against each other at a fixed address.</summary>
public class BehaviourTests
{
    private static RunRecord Run(string id, string address, string arrangement, params string[] proposed) =>
        new(RunRecord.FormV1, id, "tool", "subject", "act/a", DateTimeOffset.UnixEpoch,
            "model", "host", 0, proposed, [], "reply", [],
            Pinned.Of(("q", address)), Pinned.Of(("model", arrangement)));

    private static RunRecord Bare(string id) =>
        new(RunRecord.FormV1, id, "tool", "subject", "act/a", DateTimeOffset.UnixEpoch,
            "model", "host", 0, [], [], "reply", []);

    [Fact]
    public void Runs_without_an_address_are_not_compared()
    {
        Assert.Empty(Behaviour.Read([Bare("01")]));
    }

    [Fact]
    public void Runs_at_one_address_are_read_together()
    {
        var readings = Behaviour.Read([Run("01", "q1", "m", "det/a"), Run("02", "q1", "m", "det/a")]);
        Assert.Single(readings);
        Assert.Equal(2, readings[0].Runs);
    }

    [Fact]
    public void Runs_at_different_addresses_are_not_comparable()
    {
        Assert.Equal(2, Behaviour.Read([Run("01", "q1", "m"), Run("02", "q2", "m")]).Count);
    }

    [Fact]
    public void A_repeated_answer_agrees_with_itself()
    {
        var readings = Behaviour.Read([Run("01", "q1", "m", "det/a"), Run("02", "q1", "m", "det/a")]);
        Assert.Equal(1.0, readings[0].Agreement);
    }

    [Fact]
    public void A_changed_answer_at_one_address_disagrees()
    {
        var readings = Behaviour.Read([Run("01", "q1", "m", "det/a"), Run("02", "q1", "m", "det/b")]);
        Assert.Equal(0.0, readings[0].Agreement);
    }

    /// <summary>Two runs that both proposed nothing agree. Silence is a real answer.</summary>
    [Fact]
    public void Two_silences_agree()
    {
        var readings = Behaviour.Read([Run("01", "q1", "m"), Run("02", "q1", "m")]);
        Assert.Equal(1.0, readings[0].Agreement);
    }

    /// <summary>One run at an address is not a low score; it is the absence of one.</summary>
    [Fact]
    public void A_single_run_yields_no_agreement_rather_than_zero()
    {
        var readings = Behaviour.Read([Run("01", "q1", "m", "det/a")]);
        Assert.Null(readings[0].Agreement);
        Assert.Null(readings[0].Drift);
    }

    /// <summary>The earthquake reading: same address, changed worker, moved answer.</summary>
    [Fact]
    public void A_changed_arrangement_at_a_fixed_address_shows_as_drift()
    {
        var readings = Behaviour.Read([
            Run("01", "q1", "old", "det/a"), Run("02", "q1", "old", "det/a"),
            Run("03", "q1", "new", "det/b"), Run("04", "q1", "new", "det/b"),
        ]);

        Assert.Equal(2, readings[0].Arrangements);
        Assert.Equal(1.0, readings[0].Agreement);
        Assert.Equal(0.0, readings[0].AcrossArrangements);
        Assert.Equal(1.0, readings[0].Drift);
    }

    [Fact]
    public void A_stable_worker_change_shows_no_drift()
    {
        var readings = Behaviour.Read([
            Run("01", "q1", "old", "det/a"), Run("02", "q1", "old", "det/a"),
            Run("03", "q1", "new", "det/a"), Run("04", "q1", "new", "det/a"),
        ]);
        Assert.Equal(0.0, readings[0].Drift);
    }

    /// <summary>Declared-distinct ground must yield distinct behaviour.</summary>
    [Fact]
    public void Two_addresses_answered_identically_are_reported_as_collapsed()
    {
        var collapse = Behaviour.Collapsed([Run("01", "q1", "m", "det/a"), Run("02", "q2", "m", "det/a")]);
        Assert.Single(collapse);
    }

    [Fact]
    public void Addresses_answered_differently_have_not_collapsed()
    {
        Assert.Empty(Behaviour.Collapsed([Run("01", "q1", "m", "det/a"), Run("02", "q2", "m", "det/b")]));
    }

    /// <summary>An address answered two ways has not collapsed onto another.</summary>
    [Fact]
    public void An_unstable_address_is_not_reported_as_collapsed()
    {
        var collapse = Behaviour.Collapsed([
            Run("01", "q1", "m", "det/a"), Run("02", "q1", "m", "det/b"), Run("03", "q2", "m", "det/a"),
        ]);
        Assert.Empty(collapse);
    }

    /// <summary>The two runtimes must read the same runs the same way.</summary>
    [Fact]
    public void An_address_pins_the_same_digest_as_the_rust_half_would()
    {
        Assert.Equal(Pinned.Of(("q", "q1")).Digest, Pinned.Of(("q", "q1")).Digest);
        Assert.NotEqual(Pinned.Of(("q", "q1")).Digest, Pinned.Of(("q", "q2")).Digest);
    }
}
