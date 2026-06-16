<details>
<summary>XSD contract: <code>OccurenceType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:simpleType name="OccurenceType">
		<xs:annotation>
			<xs:documentation>OccurenceType is used to express the maximum occurrence of an object. It combines an integer, equal or greater than 1, and the literal text, "unbounded", for objects which have no upper limit on its occurrence.</xs:documentation>
		</xs:annotation>
		<xs:union memberTypes="MaxOccursNumberType UnboundedCodeType"/>
	</xs:simpleType>
```

</details>
